#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_imports)]

use anyhow::Result;
use iota_client::block::payload::{Payload, TaggedDataPayload};
use iota_client::block::{BlockId, Block};
use iota_client::Client;
use pyo3::prelude::*;

//thread, time::Duration,
use termion::{event::Key, input::TermRead};

use std::{
    fs,
    error::Error,
    str::FromStr,
    pin::Pin,
    path::Path,
    io::{stdin, stdout, Write},
    collections::HashSet,
    process::exit,
};
use futures::Future;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct GameInfo {
    game_id: String,
    game_name: String,
    parent: BlockId,
}
fn get_latest_64p_game_info() -> Result<(String, String)> {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let module = PyModule::from_code(py, include_str!("get_np_games.py"), "np_games.py", "np_games")?;
    let function = module.getattr("get_latest_64p_game_info")?;
    let result = function.call0()?;
    let result = result.extract::<(String, String)>()?;

    Ok(result)
}

//Store last block to make a linked list
fn get_last_block_id() -> Result<Option<BlockId>> {
    let block_file_path = Path::new("last_block_id.txt");

    if block_file_path.exists() {
        let block_id_string = fs::read_to_string(block_file_path)?;
        let block_id = BlockId::from_str(&block_id_string)?;
        Ok(Some(block_id))
    } else {
        Ok(None)
    }
}
fn set_last_block_id(block: BlockId) -> Result<()>{
    let block_file_path = Path::new("last_block_id.txt");
    fs::write(block_file_path, &block.to_string())?;
    Ok(())
}

//Store Mnemonic locally; This is essentailly a server session ID
fn get_or_create_mnemonic() -> Result<String> {
    let mnemonic_file_path = Path::new("mnemonic.txt");

    if mnemonic_file_path.exists() {
        let mnemonic = fs::read_to_string(mnemonic_file_path)?;
        Ok(mnemonic)
    } else {
        let mnemonic = Client::generate_mnemonic()?;
        fs::write(mnemonic_file_path, &mnemonic)?;
        Ok(mnemonic)
    }
}


fn get_game_info(client: &Client, block: &Block) -> Pin<Box<dyn Future<Output = Result<Vec<GameInfo>, Box<dyn Error>>>>> {
    Box::pin(async move {
        let mut game_infos: Vec<GameInfo> = Vec::new();

        match block.payload() {
            Some(Payload::TaggedData(payload)) => {
                let game_info_string = String::from_utf8(payload.clone().data().to_vec()).expect("found invalid UTF-8");
                let game_info: GameInfo = serde_json::from_str(&game_info_string).unwrap();
                println!("Found {} {}", game_info.game_name, game_info.game_id);
                let parent: Block = client.get_block(&game_info.parent).await?;

                // Recursive step
                game_infos.extend(get_game_info(&client, &parent).await?);
                game_infos.push(game_info);
            }
            None => {
                println!("No more data.");
            }
        }
        Ok(game_infos)
    })
}

/*
//Get data from ledger
async fn retrieve_unique_payloads_from_blocks(client: &Client, block_ids: &[BlockId]) -> Result<HashSet<(String, String)>> {
    let mut unique_payloads = HashSet::new();

    for block_id in block_ids {
        let block = client.get_block(block_id).await?;
        if let Some(Payload::TaggedData(payload)) = block.payload() {
            let tag = String::from_utf8(payload.tag().to_vec()).expect("found invalid UTF-8");
            let data = String::from_utf8(payload.data().to_vec()).expect("found invalid UTF-8");

            // Insert the payload into the HashSet (unique payloads)
            unique_payloads.insert((tag, data));
        }
    }

    Ok(unique_payloads)
}
*/
/*
//Get the block ids from an IOTA address
async fn fetch_uploaded_block_ids(client: &Client) -> Result<Vec<BlockId>> {
    let mut block_ids = Vec::new();

    let output = client.finish().await?;
    for output_id in output.iter() {
        let current_block_id: BlockId = BlockId::from_str(&output_id)?;
        let block = client.get_block(&current_block_id).await?;
        if let Some(Payload::TaggedData(payload)) = block.payload() {
            let tag = String::from_utf8(payload.tag().to_vec()).expect("found invalid UTF-8");
            if tag == "Block ID" {
                let data = String::from_utf8(payload.data().to_vec()).expect("found invalid UTF-8");
                let block_id = BlockId::from_str(&data)?;
                block_ids.push(block_id);
            }
        }
    }

    Ok(block_ids)
}
*/

fn get_user_input() -> Option<Key> {
    let stdin = stdin();
    let mut stdin_keys = stdin.keys();
    let _key = stdin_keys.next();

    _key?.ok()
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::builder()
        .with_node("https://api.testnet.shimmer.network")?
        .finish()?;

    //Python interpreter
    pyo3::prepare_freethreaded_python();

    //Store block ids on ledger in an address
    let mut last_block_id: BlockId;
    let mut last_block: Block;
    match get_last_block_id()?{
        Some(block_id)=> { 
            last_block_id = block_id;
            println!("Fetching last block");
            last_block = client.get_block(&last_block_id).await?;
        }
        None => {
            println!("No start block found: Creating one...");
            last_block = client.block().finish().await?;
            last_block_id = last_block.id();
            set_last_block_id(last_block_id)?;
        }
    }
    println!("Starting block {}",last_block_id);
    //Store uploaded block ids
    let mut uploaded_block_ids = Vec::new();
    uploaded_block_ids.push(last_block_id);
    loop {
        println!("Fetching Latest NP Game ID");
        //Get game info from python
        let (game_name, game_id) = get_latest_64p_game_info()?;

        // Create a custom payload.
        let game_info = GameInfo {
            game_id: game_id.clone(),
            game_name: game_name.clone(),
            parent: last_block_id,
        };
        let tagged_data_payload = TaggedDataPayload::new(
            format!("NP_GAME_CACHE").into_bytes(),
            serde_json::to_string_pretty(&game_info).unwrap().into_bytes(),
        )?;

        println!("Uploading Game ID {}",game_id);
        // Send a block with the gamedata payload.
        let block = client
            .block()
            .finish_block(Some(Payload::from(tagged_data_payload)))
            .await?;

        //Now Store the last block for Retrieval
        set_last_block_id(block.id())?;
        last_block = block.clone();
        last_block_id = last_block.id();

        println!(
            "Sent block with custom payload for game: {} ({}), Block ID: {}",
            game_name, game_id, block.id()
        );

        println!("Saving Block ID");
        uploaded_block_ids.push(block.id());
        //Send Block ID into address
        let block_id_payload = TaggedDataPayload::new(
            format!("Block ID").into_bytes(),
            format!("{}", block.id()).into_bytes(),
        )?;
        
        let _block_with_block_id = client
            .block()
            .finish_block(Some(Payload::from(block_id_payload)))
            .await?;
        
        print!("Press SPACE to fetch again or ENTER to retrieve all data: ");
        stdout().flush().unwrap();


        
        println!("Retrieving all data...");
        let game_infos: Vec<GameInfo> = get_game_info(&client,&last_block).await.unwrap();
        
        println!("All unique data:");
        for game_info in game_infos {
            println!("ID: {}, Name: {}", game_info.game_id, game_info.game_name);
        }
        
        break;
    }

    Ok(())
}
