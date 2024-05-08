use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct ContractAddresses {
    pub proxy: String,
}

const RELATIVE_ADDRESS_FILE_PATH: &str = "src/addresses.json";

pub fn get_contract_addresses() -> ContractAddresses {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(RELATIVE_ADDRESS_FILE_PATH);
    let addresses_json = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&addresses_json).unwrap()
}

pub fn set_contract_addresses(addresses: ContractAddresses) {
    let json_str = serde_json::to_string(&addresses).unwrap();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(RELATIVE_ADDRESS_FILE_PATH);
    std::fs::write(path, json_str).unwrap();
}
