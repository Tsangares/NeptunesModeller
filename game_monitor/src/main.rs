use anyhow::Result;
use iota_client::block::payload::{Payload, TaggedDataPayload};
use iota_client::block::{BlockId, Block};
use iota_client::Client;
use iota_client::node_manager::node::NodeAuth;
use serde::{Deserialize, Serialize};
use futures::Future;
use pyo3::prelude::*;
use std::{
    fs,
    error::Error,
    str::FromStr,
    pin::Pin,
    path::Path,
    io::{stdout, Write},
};
#[allow(unused_imports)]
use std::process::exit;

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
#[allow(dead_code)]
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


fn get_game_info<'a>(client: &'a Client, block: Block) -> Pin<Box<dyn Future<Output = Result<Vec<GameInfo>, Box<dyn Error + 'a>>> + 'a>> {    Box::pin(async move {
        let mut game_infos: Vec<GameInfo> = Vec::new();

        match block.payload() {
            Some(Payload::TaggedData(payload)) => {
                let game_info_string = String::from_utf8(payload.clone().data().to_vec()).expect("found invalid UTF-8");
                let game_info: GameInfo = serde_json::from_str(&game_info_string).unwrap();
                println!("Found {} {}", game_info.game_name, game_info.game_id);
                let parent: Block = client.get_block(&game_info.parent).await?;

                // Recursive step
                game_infos.extend(get_game_info(&client, parent).await?);
                game_infos.push(game_info);
            }
            None => {
                println!("No more data.");
            }
            _ => {
                println!("Payload type not supported.");
            }
        }
        Ok(game_infos)
    })
}

async fn get_node_info(url: &str) -> () {
    let response = Client::get_node_info(&url,None).await;
    let node_info: String = serde_json::to_string_pretty(&response).unwrap();
    println!("{}",node_info);
}

#[allow(dead_code)]
fn get_node_list() -> Result<Vec<String>> {
    let node_list_file_path = Path::new("node_list.json");
    if node_list_file_path.exists() {
        let node_list_text = fs::read_to_string(node_list_file_path)?;
        let node_list: Vec<String> = serde_json::from_str(&node_list_text).unwrap();
        Ok(node_list)
    } else {
        let node_list: Vec<String> = [
            "https://api.shimmer.network",
            "https://shimmer.iotatangle.us"
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        Ok(node_list)
    }
}

fn get_cpu_limit() -> usize {
    let mut cpus = num_cpus::get();
    if cpus > 1 {
        cpus = cpus-1;
    }
    cpus
}

#[tokio::main]
async fn main() -> Result<()> {
    //TODO: Load node slices if you can't find remote ones. 
    //let nodes = get_node_list()?;
    //let nodes_slice: &[&str] = &nodes.iter().map(|s| s.as_str()).collect::<Vec<_>>()[..];
    let cpus = get_cpu_limit();

    println!("Connecting to Shimmer Network.");
    let client = Client::builder()
        .with_primary_node("https://api.shimmer.network",None).expect("Public node down.")
        .with_primary_pow_node("https://shimmer.iotatangle.us",None).expect("Pow node down.")
        .with_node("https://multiverse.dlt.builders").expect("Backup node down.")
        //.with_nodes(&nodes_slice)?
        .with_local_pow(false)
        .finish()?;

    //Python interpreter
    pyo3::prepare_freethreaded_python();

    //Store block ids on ledger in an address
    let mut last_block: Block;
    match get_last_block_id()? {
        Some(block_id)=> {
            println!("Fetching last block");
            last_block = client.get_block(&block_id).await?;
        }
        None => {
            println!("No start block found: Creating one...");
            last_block = client.block().finish().await?;
            set_last_block_id(last_block.id())?;
        }
    }
    println!("Starting block {}",last_block.id());
    //Store uploaded block ids
    let mut uploaded_block_ids = Vec::new();
    uploaded_block_ids.push(last_block.id());
    loop {
        println!("Fetching Latest NP Game ID");
        //Get game info from python
        let (game_name, game_id) = get_latest_64p_game_info()?;

        // Create a custom payload.
        let game_info = GameInfo {
            game_id: game_id.clone(),
            game_name: game_name.clone(),
            parent: last_block.id(),
        };
        let tagged_data_payload = TaggedDataPayload::new(
            format!("NP_GAME_CACHE").into_bytes(),
            serde_json::to_string_pretty(&game_info).unwrap().into_bytes(),
        )?;

        println!("Searching for Nonce to upload Game ID {}",game_id);
        // Send a block with the gamedata payload.
        let block = client
            .block()
            .finish_block(Some(Payload::from(tagged_data_payload)))
            .await?;

        //Now Store the last block for Retrieval
        set_last_block_id(block.id())?;
        last_block = block.clone();

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
                
        println!("Retrieving all data...");
        let game_infos: Vec<GameInfo> = get_game_info(&client,last_block.clone()).await.unwrap();
        
        println!("All unique data:");
        for game_info in game_infos {
            println!("ID: {}, Name: {}", game_info.game_id, game_info.game_name);
        }
        
        break;
    }

    Ok(())
}
