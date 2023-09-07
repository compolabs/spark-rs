use fuels::accounts::predicate::Predicate;
use fuels::prelude::ViewOnlyAccount;
use fuels::types::{Address, Bits256};
use spark_sdk::limit_orders_utils::limit_orders_interactions::cancel_order;
use spark_sdk::{
    limit_orders_utils::{
        limit_orders_interactions::create_order, LimitOrderPredicateConfigurables,
    },
    proxy_utils::{deploy_proxy_contract, ProxySendFundsToPredicateParams},
};
use src20_sdk::{deploy_token_factory_contract, token_factory_abi_calls};

use crate::utils::cotracts_utils::token_utils::deploy_tokens;
use crate::utils::local_tests_utils::init_wallets;
use crate::utils::print_title;

#[tokio::test]
async fn recreate_order_test() {
    print_title("Recreate Order Test");
    //--------------- WALLETS ---------------
    let wallets = init_wallets().await;
    let admin = &wallets[0];
    let alice = &wallets[1];
    let alice_address = Address::from(alice.address());

    println!("alice_address = 0x{:?}\n", alice_address);
    //--------------- TOKENS ---------------
    let factory =
        deploy_token_factory_contract(admin, "tests/artefacts/factory/token-factory.bin").await;
    let assets = deploy_tokens(&factory, "tests/artefacts/tokens.json").await;

    let usdc = assets.get("USDC").unwrap();
    let uni = assets.get("UNI").unwrap();

    let amount0 = 1_000_000_000; //1000 USDC
    let amount1 = 200_000_000_000; // 200 UNI
    println!("USDC AssetId (asset0) = 0x{:?}", usdc.asset_id);
    println!("UNI AssetId (asset1) = 0x{:?}", uni.asset_id);
    println!("amount0 = {:?} USDC", amount0 / 1_000_000);
    println!("amount1 = {:?} UNI", amount1 / 1_000_000_000);

    let price_decimals = 9;
    let exp = price_decimals + usdc.decimals - uni.decimals;
    let price = amount1 * 10u64.pow(exp as u32) / amount0;
    println!("Price = {:?} UNI/USDC", price);

    token_factory_abi_calls::mint(&factory, alice_address, &usdc.symbol, amount0)
        .await
        .unwrap();
    let initial_alice_usdc_balance = alice.get_asset_balance(&usdc.asset_id).await.unwrap();
    println!("Alice minting {:?} USDC\n", amount0 / 1000_000);

    //--------------- PREDICATE ---------

    let configurables = LimitOrderPredicateConfigurables::new()
        .with_ASSET0(usdc.bits256)
        .with_ASSET1(uni.bits256)
        .with_ASSET0_DECIMALS(usdc.decimals as u8)
        .with_ASSET1_DECIMALS(uni.decimals as u8)
        .with_MAKER(Bits256::from_hex_str(&alice.address().hash().to_string()).unwrap())
        .with_PRICE(price)
        .with_MIN_FULFILL_AMOUNT0(amount0);

    let predicate: Predicate =
        Predicate::load_from("./limit-order-predicate/out/debug/limit-order-predicate.bin")
            .unwrap()
            .with_configurables(configurables)
            .with_provider(admin.provider().unwrap().clone());
    println!("Predicate root = {:?}\n", predicate.address());
    //--------------- THE TEST ---------
    assert!(alice.get_asset_balance(&usdc.asset_id).await.unwrap() == amount0);
    let proxy = deploy_proxy_contract(&alice, "proxy-contract/out/debug/proxy-contract.bin").await;
    let params = ProxySendFundsToPredicateParams {
        predicate_root: predicate.address().into(),
        asset_0: usdc.bits256,
        asset_1: uni.bits256,
        maker: alice_address,
        min_fulfill_amount_0: 1,
        price,
        asset_0_decimals: 6,
        asset_1_decimals: 9,
        price_decimals: 9,
    };
    let proxy_address = format!("0x{}", proxy.contract_id().hash);
    println!("proxyAddress = {:?}", proxy_address);
    create_order(&alice, &proxy_address, params.clone(), amount0 / 2)
        .await
        .unwrap();
    println!("Alice transfers 500 USDC to predicate\n");

    create_order(&alice, &proxy_address, params.clone(), amount0 / 2)
        .await
        .unwrap();
    println!("Alice transfers 500 USDC to predicate\n");

    cancel_order(&alice, &predicate, usdc.asset_id, amount0)
        .await
        .unwrap();

    println!("Alice canceles the order\n");
    // The predicate root's coin has been spent
    let predicate_balance = predicate.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(predicate_balance, 0);

    // Wallet balance is the same as before it sent the coins to the predicate
    let wallet_balance = alice.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(wallet_balance, initial_alice_usdc_balance);
    println!("Alice balance 1000 UDSC\n");
}
