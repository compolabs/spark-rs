use fuels::accounts::wallet::WalletUnlocked;
use std::collections::HashMap;

use fuels::types::{AssetId, Bits256};
use serde::Deserialize;
use src20_sdk::{token_factory_abi_calls, TokenFactoryContract};

pub struct Asset {
    pub bits256: Bits256,
    pub asset_id: AssetId,
    pub decimals: u64,
    pub symbol: String,
}

#[derive(Deserialize)]
pub struct TokenConfig {
    pub asset_id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u64,
}

pub async fn deploy_tokens(
    factory: &TokenFactoryContract<WalletUnlocked>,
    tokens_json_path: &str,
) -> HashMap<String, Asset> {
    let tokens_json = std::fs::read_to_string(tokens_json_path).unwrap();
    let token_configs: Vec<TokenConfig> = serde_json::from_str(&tokens_json).unwrap();

    let mut assets: HashMap<String, Asset> = HashMap::new();

    for config in token_configs {
        let name = config.name;
        let symbol = config.symbol;
        let decimals = config.decimals;

        token_factory_abi_calls::deploy(&factory, &symbol, &name, decimals)
            .await
            .unwrap();

        let bits256 = token_factory_abi_calls::asset_id(&factory, &symbol)
            .await
            .unwrap()
            .value;

        assets.insert(
            symbol.clone(),
            Asset {
                bits256,
                asset_id: AssetId::from(bits256.0),
                decimals,
                symbol,
            },
        );
    }
    assets
}

pub async fn load_tokens(tokens_json_path: &str) -> HashMap<String, Asset> {
    let tokens_json = std::fs::read_to_string(tokens_json_path).unwrap();
    let token_configs: Vec<TokenConfig> = serde_json::from_str(&tokens_json).unwrap();

    let mut assets: HashMap<String, Asset> = HashMap::new();

    for config in token_configs {
        let bits256 = Bits256::from_hex_str(&config.asset_id).unwrap();
        assets.insert(
            config.symbol.clone(),
            Asset {
                bits256,
                asset_id: AssetId::from(bits256.0),
                decimals: config.decimals,
                symbol: config.symbol.clone(),
            },
        );
    }
    assets
}
