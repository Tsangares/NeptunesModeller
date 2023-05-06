use anyhow::Result;
use iota_client::{
    block::BlockId,
    block::payload::{Payload, TaggedDataPayload},
    Client,
};
use pyo3::prelude::*;
use std::{
    io::{stdin, stdout, Write},
    collections::HashSet,
};
//thread, time::Duration,
use termion::{event::Key, input::TermRead};

fn get_latest_64p_game_info() -> Result<(String, String)> {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let module = PyModule::from_code(py, include_str!("get_np_games.py"), "np_games.py", "np_games")?;
    let function = module.getattr("get_latest_64p_game_info")?;
    let result = function.call0()?;
    let result = result.extract::<(String, String)>()?;

    Ok(result)
}

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


fn get_user_input() -> Option<Key> {
    let stdin = stdin();
    let mut stdin_keys = stdin.keys();
    let key = stdin_keys.next();

    key?.ok()
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::builder()
        .with_node("https://api.testnet.shimmer.network")?
        .finish()?;

    //Python interpreter
    pyo3::prepare_freethreaded_python();

    //Store uploaded block ids
    let mut uploaded_block_ids = Vec::new();
    loop {
        //Get game info from python
        let (game_name, game_id) = get_latest_64p_game_info()?;

        // Create a custom payload.
        let tagged_data_payload = TaggedDataPayload::new(
            format!("Game: {}", game_name).into_bytes(),
            format!("Game ID: {}", game_id).into_bytes(),
        )?;

        // Send a block with the gamedata payload.
        let block = client
            .block()
            .finish_block(Some(Payload::from(tagged_data_payload)))
            .await?;

        //Now Store the Block for Retrieval
        let block_id = block.id();
        uploaded_block_ids.push(block_id);

        println!(
            "Sent block with custom payload for game: {} ({}), Block ID: {}",
            game_name, game_id, block_id
        );
        
        print!("Press SPACE to fetch again or ENTER to retrieve all data: ");
        stdout().flush().unwrap();

        let key = get_user_input();

        match key {
            Some(Key::Char(' ')) => {
                println!("Fetching again...");
                continue;
            }
            Some(Key::Char('\n')) => {
                println!("Retrieving all data...");
                let unique_payloads = retrieve_unique_payloads_from_blocks(&client, &uploaded_block_ids).await?;

                println!("All unique data:");
                for (tag, data) in unique_payloads {
                    println!("Tag: {}, Data: {}", tag, data);
                }
                break;
            }
            Some(Key::Ctrl('c')) => {
                println!("Ctrl+C pressed. Quitting...");
                break;
            }
            _ => {
                println!("Invalid input. Exiting...");
                break;
            }
        }
    }

    Ok(())
}
