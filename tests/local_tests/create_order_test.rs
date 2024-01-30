use fuels::{
    accounts::{predicate::Predicate, Account},
    prelude::ViewOnlyAccount,
    types::{transaction::TxPolicies, Address},
};
use spark_sdk::limit_orders_utils::LimitOrderPredicateConfigurables;

use crate::utils::{
    cotracts_utils::token_utils::{deploy_token_contract, Asset},
    local_tests_utils::init_wallets,
    print_title,
};

#[tokio::test]
async fn create_order_test() {
    print_title("Create Order Test");
    //--------------- WALLETS ---------------
    let wallets = init_wallets().await;
    let admin = &wallets[0];
    let alice = &wallets[1];
    let alice_address = Address::from(alice.address());

    println!("alice_address = 0x{:?}\n", alice_address);
    //--------------- TOKENS ---------------
    let token_contarct = deploy_token_contract(&admin).await;
    let usdc = Asset::new(admin.clone(), token_contarct.contract_id().into(), "USDC");
    let btc = Asset::new(admin.clone(), token_contarct.contract_id().into(), "BTC");

    let amount0 = 40_000_000_000; //40k USDC
    let amount1 = 1_00_000_000; // 1 BTC
    println!("USDC AssetId (asset0) = 0x{:?}", usdc.asset_id);
    println!("BTC AssetId (asset1) = 0x{:?}", btc.asset_id);
    println!("amount0 = {:?} USDC", amount0 / 1_000_000);
    println!("amount1 = {:?} BTC", amount1 / 1_00_000_000);

    let price_decimals = 9;
    let exp = price_decimals + usdc.decimals - btc.decimals;
    let price = amount0 * 10u64.pow(exp as u32) / amount1;
    println!("Price = {:?} BTC/USDC", price / 1_000_000_000);

    usdc.mint(alice_address, amount0).await.unwrap();
    println!("Alice minting {:?} USDC", amount0 / 1_000_000);

    //--------------- PREDICATE ---------

    let configurables = LimitOrderPredicateConfigurables::new()
        .with_ASSET0(usdc.asset_id.into())
        .with_ASSET1(btc.asset_id.into())
        .with_ASSET0_DECIMALS(usdc.decimals as u8)
        .with_ASSET1_DECIMALS(btc.decimals as u8)
        .with_MAKER(alice.address().into())
        .with_PRICE(price)
        .with_MIN_FULFILL_AMOUNT0(amount0);

    let predicate: Predicate =
        Predicate::load_from("./limit-order-predicate/out/debug/limit-order-predicate.bin")
            .unwrap()
            .with_configurables(configurables)
            .with_provider(alice.provider().unwrap().clone());

    println!("Predicate root = {:?}\n", predicate.address());
    //--------------- THE TEST ---------
    assert!(alice.get_asset_balance(&usdc.asset_id).await.unwrap() == amount0);

    let policies = TxPolicies::default().with_gas_price(1);
    alice
        .transfer(predicate.address(), amount0, usdc.asset_id, policies)
        .await
        .unwrap();

    assert!(alice.get_asset_balance(&usdc.asset_id).await.unwrap() == 0);
    assert!(predicate.get_asset_balance(&usdc.asset_id).await.unwrap() == amount0);
}
