use anyhow::Result;
use iota_client::block::payload::{Payload, TaggedDataPayload};
use iota_client::block::{BlockId, Block};
use iota_client::{
    mqtt::{MqttEvent, MqttPayload, Topic},
    Client,
};
#[allow(unused_imports)]
use iota_client::secret::SecretManager;
#[allow(unused_imports)]
use iota_client::api::GetAddressesBuilder;
#[allow(unused_imports)]
use iota_client::block::output::BasicOutputBuilder;
#[allow(unused_imports)]
use iota_client::block::output::feature::*;
#[allow(unused_imports)]
use iota_client::block::output::Feature;
#[allow(unused_imports)]
use iota_client::block::unlock::SignatureUnlock;
#[allow(unused_imports)]
use iota_client::node_manager::node::NodeAuth;
use serde::{Deserialize, Serialize};
use futures::Future;
use std::sync::{mpsc::channel, Arc, Mutex};
use std::{
    fs,
    error::Error,
    str::FromStr,
    pin::Pin,
    path::Path,
    //io::{stdout, Write},
};
#[allow(unused_imports)]
use std::process::exit;

mod neptunes_pride;
mod shimmer;

//Maybe have a function to cast a payload to this?
#[derive(Serialize, Deserialize, Debug)]
struct GameInfo {
    game_id: String,
    game_name: String,
    parent: BlockId,
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


//** Linked List Collector **\\
//Fetching game info from client.
fn get_game_info<'a>(client: &'a Client, block: Block) -> Pin<Box<dyn Future<Output = Result<Vec<GameInfo>, Box<dyn Error + 'a>>> + 'a>> {    Box::pin(async move {
        let mut game_infos: Vec<GameInfo> = Vec::new();

        match block.payload() {
            Some(Payload::TaggedData(payload)) => {
                let game_info: GameInfo = shimmer::get_json_payload(payload);
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



#[tokio::main]
async fn main() -> Result<()> {
    println!("Connecting to Shimmer Network.");
    let client = shimmer::get_client();
    let mnemonic = shimmer::get_or_create_mnemonic();
    let secret_manager = SecretManager::try_from_mnemonic(&mnemonic)?;
    let addresses = GetAddressesBuilder::new(&secret_manager)
        .with_client(&client)
        .with_account_index(0)
        .get_raw().await?;
    let address_strings = GetAddressesBuilder::new(&secret_manager)
        .with_client(&client)
        .with_account_index(0)
        .finish().await?;
    println!("{:?}",addresses[0]);
    let output_address = address_strings.clone();
    let inputs = client.find_inputs(address_strings, 0).await?;
    let mut transaction_builder = client.block();
    for input in inputs {
        transaction_builder = transaction_builder.with_input(input)?;
    }
    let prepared_transaction = transaction_builder
        .with_output(&output_address[0].as_str(), 0)
        .await?
        .prepare_transaction()
        .await?;

    let sender = IssuerFeature::new(addresses[0]);
    let metadata = MetadataFeature::new(format!("NP_GAME_CACHE").into_bytes())?;
    let output = BasicOutputBuilder::new_with_amount(0)?
        .add_feature(Feature::from(sender))
        .add_feature(Feature::from(metadata))
        .finish_output(0)?;
    exit(10);
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
        let (game_name, game_id) = neptunes_pride::get_latest_64p_game_info()?;

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
