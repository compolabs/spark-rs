use fuels::{
    prelude::{Provider, WalletUnlocked},
    types::ContractId,
};
use spark_sdk::{
    constants::{RPC, TOKEN_CONTRACT_ID},
    spark_utils::Spark,
    print_title,
    utils::{set_contract_addresses, ContractAddresses},
};
use src20_sdk::token_utils::{Asset, TokenContract};
use std::str::FromStr;

const BASE_ASSET: &str = "BTC";
const QUOTE_ASSET: &str = "USDC";

// 🏁 Start_block: 11266711

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    print_title("✨ Deploy proxy ✨ ");

    let provider = Provider::connect(RPC).await.unwrap();

    let admin_pk = std::env::var("ADMIN").unwrap().parse().unwrap();
    let admin = &WalletUnlocked::new_from_private_key(admin_pk, Some(provider.clone()));

    let token_contract = TokenContract::new(
        &ContractId::from_str(TOKEN_CONTRACT_ID).unwrap().into(),
        admin.clone(),
    );
    let base_asset = Asset::new(
        admin.clone(),
        token_contract.contract_id().into(),
        BASE_ASSET,
    );
    let quote_asset = Asset::new(
        admin.clone(),
        token_contract.contract_id().into(),
        QUOTE_ASSET,
    );

    let block = provider.latest_block_height().await.unwrap();
    println!("🏁 Start_block: {block}\n");
    let spark = Spark::deploy_proxy(admin, &base_asset, &quote_asset).await;

    println!("Market = {:?} / {:?}", BASE_ASSET, QUOTE_ASSET);
    println!("proxy = 0x{:?}", spark.proxy.contract_id().hash);
    println!("proxy = {:?}\n", spark.proxy.contract_id().to_string());

    set_contract_addresses(ContractAddresses {
        proxy: spark.proxy.contract_id().hash.to_string(),
    });
}
