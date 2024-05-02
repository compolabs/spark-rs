use std::{env, str::FromStr};

use dotenv::dotenv;
use fuels::{
    accounts::{predicate::Predicate, wallet::WalletUnlocked},
    prelude::Provider,
    types::{Address, ContractId},
};
use spark_sdk::{
    constants::{RPC, TOKEN_CONTRACT_ID},
    limit_orders_utils::{
        limit_orders_interactions::{cancel_order, create_order},
        LimitOrderPredicateConfigurables,
    },
    print_title,
};
use src20_sdk::token_utils::{Asset, TokenContract};

// You want to buy 1 btc for 40k
const AMOUNT_0: u64 = 40_000;
const ASSET_0: &str = "USDC";

const AMOUNT_1: u64 = 1; // BTC
const ASSET_1: &str = "BTC";

#[tokio::main]
async fn main() {
    print_title("Cancel Order script");
    dotenv().ok();

    //--------------- WALLETS ---------------
    let provider = Provider::connect(RPC).await.unwrap();

    let admin_pk = env::var("ADMIN").unwrap().parse().unwrap();
    let admin = WalletUnlocked::new_from_private_key(admin_pk, Some(provider.clone()));

    let maker_pk = env::var("ALICE").unwrap().parse().unwrap();
    let maker = WalletUnlocked::new_from_private_key(maker_pk, Some(provider.clone()));
    let maker_address = Address::from(maker.address());

    println!("maker address = 0x{:?}\n", maker_address);
    //--------------- TOKENS ---------------
    let token_contract = TokenContract::new(
        &ContractId::from_str(TOKEN_CONTRACT_ID).unwrap().into(),
        admin.clone(),
    );
    let asset0 = Asset::new(admin.clone(), token_contract.contract_id().into(), ASSET_0);
    let asset1 = Asset::new(admin.clone(), token_contract.contract_id().into(), ASSET_1);

    let amount0 = asset0.parse_units(AMOUNT_0 as f64) as u64;
    let amount1 = asset1.parse_units(AMOUNT_1 as f64) as u64;
    println!("amount0 = {AMOUNT_0} {ASSET_0} ({:?})", asset0.asset_id);
    println!("amount1 = {AMOUNT_1} {ASSET_1} ({:?})", asset1.asset_id);

    let price_decimals = 9;
    let exp = price_decimals + asset0.decimals - asset1.decimals;
    let price = amount1 * 10u64.pow(exp as u32) / amount0;
    println!("price = {:?} {ASSET_0}/{ASSET_1}", price / 1_000_000_000);

    asset0.mint(maker_address, amount0).await.unwrap();

    //--------------- PREDICATE ---------
    let configurables = LimitOrderPredicateConfigurables::new()
        .with_ASSET0(asset0.asset_id.into())
        .with_ASSET1(asset1.asset_id.into())
        .with_ASSET0_DECIMALS(asset0.decimals as u8)
        .with_ASSET1_DECIMALS(asset1.decimals as u8)
        .with_MAKER(maker.address().into())
        .with_PRICE(price)
        .with_MIN_FULFILL_AMOUNT0(amount0);

    let predicate: Predicate =
        Predicate::load_from("./limit-order-predicate/out/debug/limit-order-predicate.bin")
            .unwrap()
            .with_configurables(configurables)
            .with_provider(admin.provider().unwrap().clone());

    println!("predicate root = {:?}\n", predicate.address());

    let res = create_order(&maker, predicate.address(), asset0.asset_id, amount0)
        .await
        .unwrap();

    println!("create order tx: {}\n", res.0.to_string());

    let res = cancel_order(&maker, &predicate, asset0.asset_id, amount0)
        .await
        .unwrap();

    println!("cancel order tx: {}\n", res.tx_id.unwrap().to_string());
}
