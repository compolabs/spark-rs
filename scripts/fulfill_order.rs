use std::{env, str::FromStr};

use dotenv::dotenv;
use fuels::{
    accounts::wallet::WalletUnlocked,
    prelude::Provider,
    types::{Address, ContractId},
};
use spark_sdk::{
    constants::{RPC, TOKEN_CONTRACT_ID},
    spark_utils::{CreateOrderEvent, Spark},
    print_title,
    utils::get_contract_addresses,
};
use src20_sdk::token_utils::Asset;

// You want to buy 1 btc for 70k
const QUOTE_AMOUNT: u64 = 70_000;
const QUOTE_ASSET: &str = "USDC";

const BASE_AMOUNT: u64 = 1; // BTC
const BASE_ASSET: &str = "BTC";

#[tokio::main]
async fn main() {
    print_title("Fulfill Order script");
    dotenv().ok();

    //--------------- WALLETS ---------------
    let provider = Provider::connect(RPC).await.unwrap();

    let admin_pk = env::var("ADMIN").unwrap().parse().unwrap();
    let admin = WalletUnlocked::new_from_private_key(admin_pk, Some(provider.clone()));

    let maker_pk = env::var("ALICE").unwrap().parse().unwrap();
    let maker = WalletUnlocked::new_from_private_key(maker_pk, Some(provider.clone()));
    let maker_address = Address::from(maker.address());

    let taker_pk = env::var("BOB").unwrap().parse().unwrap();
    let taker = WalletUnlocked::new_from_private_key(taker_pk, Some(provider.clone()));
    let taker_address = Address::from(taker.address());

    println!("maker address = 0x{:?}\n", maker_address);
    println!("taker address = 0x{:?}\n", taker_address);
    //--------------- TOKENS ---------------
    let token_contract_id = ContractId::from_str(TOKEN_CONTRACT_ID).unwrap().into();
    let quote_asset = Asset::new(admin.clone(), token_contract_id, QUOTE_ASSET);
    let base_asset = Asset::new(admin.clone(), token_contract_id, BASE_ASSET);

    let quote_amount = quote_asset.parse_units(QUOTE_AMOUNT as f64) as u64;
    let base_amount = base_asset.parse_units(BASE_AMOUNT as f64) as u64;

    let price_decimals = 9;

    let exp = price_decimals + base_asset.decimals - quote_asset.decimals;
    let price = (quote_amount as u128 * 10u128.pow(exp as u32) / base_amount as u128) as u64;

    // println!("price = {:?} {QUOTE_ASSET}/{BASE_ASSET}", price / 1e9);

    quote_asset.mint(maker_address, quote_amount).await.unwrap();
    base_asset.mint(taker_address, base_amount).await.unwrap();

    //--------------- PREDICATE ---------
    let contracts = get_contract_addresses();
    let spark = Spark::new(&admin, &contracts.proxy).await;
    let buy_predicate = spark.get_buy_predicate(&maker, &base_asset, &quote_asset, price, 1);

    let root = buy_predicate.address();

    let res = spark
        .with_account(&maker)
        .create_order(root.into(), quote_asset.asset_id, quote_amount, price)
        .await
        .unwrap();

    println!(
        "create order event: {:#?}\n",
        res.decode_logs_with_type::<CreateOrderEvent>().unwrap()
    );

    let res = spark
        .fulfill_order(
            &taker,
            &buy_predicate,
            maker.address(),
            quote_asset.asset_id,
            quote_amount,
            base_asset.asset_id,
            base_amount,
        )
        .await
        .unwrap();

    println!("fulfill order tx: {}\n", res.tx_id.unwrap().to_string());
}
