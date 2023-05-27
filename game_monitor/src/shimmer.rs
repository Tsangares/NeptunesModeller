use iota_client::Client;
use iota_client::block::payload::TaggedDataPayload;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;

//Possibly include more functions here if PoW Is not enabled and to get the list of nodes otherwise.
pub fn get_client() -> Client {
    Client::builder()
        .with_primary_node("https://api.shimmer.network",None).expect("Public node down.")
        .with_primary_pow_node("https://shimmer.iotatangle.us",None).expect("Pow node down.")
        .with_node("https://multiverse.dlt.builders").expect("Backup node down.")
        .with_local_pow(false)
        .finish().expect("Failed to get client.")
}

//Store Mnemonic locally; This is essentailly a server session ID
pub fn get_or_create_mnemonic() -> String {
    let mnemonic_file_path = Path::new("mnemonic.txt");

    if mnemonic_file_path.exists() {
        fs::read_to_string(mnemonic_file_path).expect("Couldn't write Mnemonic!")
    } else {
        let mnemonic = Client::generate_mnemonic().expect("Couldn't make Mnemonic");
        fs::write(mnemonic_file_path, &mnemonic).expect("Couldn't write Mnemonic!");
        mnemonic
    }
}

pub fn get_string_payload(payload: &Box<TaggedDataPayload>) -> String{
    String::from_utf8(payload.data().to_vec()).expect("found invalid UTF-8")
}

fn deserialize_json<T: DeserializeOwned>(json_str: &str) -> T {
    serde_json::from_str(json_str)
        .expect("Invalid JSON Data")
}

pub fn get_json_payload<T: DeserializeOwned>(payload: &Box<TaggedDataPayload>) -> T{
    let payload_string = get_string_payload(payload);
    deserialize_json(&payload_string)
}
