use std::{collections::HashMap, fs, str::FromStr};

use fuels::{
    accounts::wallet::WalletUnlocked,
    prelude::BASE_ASSET_ID,
    test_helpers::{launch_custom_provider_and_get_wallets, WalletsConfig},
    types::{AssetId, ContractId},
};

use super::cotracts_utils::token_utils::{self, Asset, DeployTokenConfig};

pub async fn init_wallets() -> Vec<WalletUnlocked> {
    let config = WalletsConfig::new(Some(5), Some(1), Some(1_000_000_000));
    launch_custom_provider_and_get_wallets(config, None, None).await
}

pub async fn init_tokens(admin: &WalletUnlocked) -> HashMap<String, Asset> {
    let deploy_config_json_str = fs::read_to_string("tests/artefacts/tokens.json")
        .expect("Should have been able to read the file");
    let deploy_configs: serde_json::Value =
        serde_json::from_str(deploy_config_json_str.as_str()).unwrap();
    let deploy_configs = deploy_configs.as_array().unwrap();
    let mut assets: HashMap<String, Asset> = HashMap::new();
    for config_value in deploy_configs {
        let config = DeployTokenConfig {
            name: String::from(config_value["name"].as_str().unwrap()),
            symbol: String::from(config_value["symbol"].as_str().unwrap()),
            decimals: config_value["decimals"].as_u64().unwrap() as u8,
            mint_amount: config_value["mint_amount"].as_u64().unwrap_or(0),
        };

        let instance = if config.symbol != "ETH" {
            Some(token_utils::deploy_token_contract(&admin, &config).await)
        } else {
            None
        };
        let contract_id = match instance {
            Option::Some(instance) => ContractId::from(instance.contract_id()),
            Option::None => ContractId::from_str(BASE_ASSET_ID.to_string().as_str())
                .expect("Cannot parse BASE_ASSET_ID to contract id"),
        };

        assets.insert(
            String::from(config_value["symbol"].as_str().unwrap()),
            Asset {
                config,
                contract_id,
                asset_id: AssetId::from(*contract_id),
                default_price: config_value["default_price"].as_u64().unwrap_or(0) * 10u64.pow(9),
                instance: Option::None,
            },
        );
    }
    assets
}
