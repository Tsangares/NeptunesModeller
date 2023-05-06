use anyhow::Result;
use iota_client::{
    block::payload::{Payload, TaggedDataPayload},
    Client, secret::SecretManager,
};
use pyo3::prelude::*;
use pyo3;
use std::{thread, time::Duration};

fn get_latest_64p_game_info() -> Result<(String, String)> {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let module = PyModule::from_code(py, include_str!("get_np_games.py"), "np_games.py", "np_games")?;
    let function = module.getattr("get_latest_64p_game_info")?;
    let result = function.call0()?;
    let result = result.extract::<(String, String)>()?;

    Ok(result)
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::builder()
        .with_node("https://api.testnet.shimmer.network")?
        .finish()?;

    let mnemonic = Client::generate_mnemonic()?;
    println!("Mnemonic: {mnemonic}");

    let secret_manager = SecretManager::try_from_mnemonic(&mnemonic)?;

    pyo3::prepare_freethreaded_python();

    loop {
        let (game_name, game_id) = get_latest_64p_game_info()?;

        let addresses = client.get_addresses(&secret_manager).finish().await?;

        println!("Generated addresses: {:#?}", addresses);

        // Create a custom payload.
        let tagged_data_payload = TaggedDataPayload::new(
            format!("Game: {}", game_name).into_bytes(),
            format!("Game ID: {}", game_id).into_bytes(),
        )?;

        // Send a block with the custom payload.
        let block = client
            .block()
            .finish_block(Some(Payload::from(tagged_data_payload)))
            .await?;

        println!(
            "Sent block with custom payload for game: {} ({}), Block ID: {}",
            game_name, game_id, block.id()
        );

        thread::sleep(Duration::from_secs(3600));
    }
}
