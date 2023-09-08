use std::{env, str::FromStr};

use dotenv::dotenv;
use fuels::{
    accounts::{predicate::Predicate, wallet::WalletUnlocked},
    prelude::{Provider, ViewOnlyAccount},
    types::{Address, Bits256, ContractId},
};
use spark_sdk::{
    limit_orders_utils::{
        limit_orders_interactions::{create_order, fulfill_order},
        LimitOrderPredicateConfigurables,
    },
    proxy_utils::ProxySendFundsToPredicateParams,
};
use src20_sdk::{token_factory_abi_calls, TokenFactoryContract};

use crate::utils::{cotracts_utils::token_utils::load_tokens, print_title};

const RPC: &str = "beta-4.fuel.network";
const PROXY_ADDRESS: &str = "0x737b9275a850fc0d2e40a2e17e6d00ef0568df95657c2726a854089cf176af15";
const FACTORY_ADDRESS: &str = "0xd8c627b9cd9ee42e2c2bd9793b13bc9f8e9aad32e25a99ea574f23c1dd17685a";
//https://spark-indexer.spark-defi.com/api/graph/compolabs/spark_indexer

#[tokio::test]
async fn fulfill_order_test() {
    print_title("Fulfill Order Test");
    dotenv().ok();

    //--------------- WALLETS ---------------
    let provider = Provider::connect(RPC).await.unwrap();

    let admin_pk = env::var("ADMIN").unwrap().parse().unwrap();
    let admin = WalletUnlocked::new_from_private_key(admin_pk, Some(provider.clone()));
    let admin_address = Address::from(admin.address());

    let alice_pk = env::var("ALICE").unwrap().parse().unwrap();
    let alice = WalletUnlocked::new_from_private_key(alice_pk, Some(provider.clone()));
    let alice_address = Address::from(alice.address());

    let bob_pk = env::var("BOB").unwrap().parse().unwrap();
    let bob = WalletUnlocked::new_from_private_key(bob_pk, Some(provider.clone()));
    let bob_address = Address::from(bob.address());

    println!("admin_address = 0x{:?}", admin_address);
    println!("alice_address = 0x{:?}", alice_address);
    println!("bob_address = 0x{:?}\n", bob_address);
    //--------------- TOKENS ---------------
    let id = ContractId::from_str(FACTORY_ADDRESS).unwrap();
    let factory = TokenFactoryContract::new(id, admin.clone());

    let assets = load_tokens("tests/artefacts/tokens.json").await;
    let usdc = assets.get("USDC").unwrap();
    let uni = assets.get("UNI").unwrap();

    let amount0 = 1000_000_000_u64; //1000 USDC
    let amount1 = 300_000_000_000_u64; //200 UNI
    println!("USDC AssetId (asset0) = 0x{:?}", usdc.asset_id);
    println!("UNI AssetId (asset1) = 0x{:?}", uni.asset_id);
    println!("amount0 = {:?} USDC", amount0 / 1000_000);
    println!("amount1 = {:?} UNI", amount1 / 1000_000_000);

    let price_decimals = 9;
    let exp = price_decimals + usdc.decimals - uni.decimals;
    let price = amount1 * 10u64.pow(exp as u32) / amount0;

    let initial_alice_usdc_balance = alice.get_asset_balance(&usdc.asset_id).await.unwrap();
    let initial_bob_uni_balance = bob.get_asset_balance(&uni.asset_id).await.unwrap();
    if initial_alice_usdc_balance < amount0 {
        token_factory_abi_calls::mint(&factory, alice_address, &usdc.symbol, amount0)
            .await
            .unwrap();
        println!("Alice minting {:?} USDC\n", amount0 / 1000_000);
    }
    if initial_bob_uni_balance < amount1 {
        token_factory_abi_calls::mint(&factory, bob_address, &uni.symbol, amount1)
            .await
            .unwrap();
        println!("Bob minting {:?} UNI\n", amount1 / 1000_000_000);
    }

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

    // let proxy = proxy_instance_by_address(&alice, &PROXY_ADDRESS);
    // println!("proxy = {:?}", proxy.contract_id().to_string());

    create_order(&alice, PROXY_ADDRESS, params, amount0)
        .await
        .unwrap();

    let initial_bob_usdc_balance = bob.get_asset_balance(&usdc.asset_id).await.unwrap();
    let initial_bob_uni_balance = bob.get_asset_balance(&uni.asset_id).await.unwrap();
    let initial_alice_uni_balance = alice.get_asset_balance(&uni.asset_id).await.unwrap();

    // The predicate root has received the coin
    let predicate_usdc_balance = predicate.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert!(predicate_usdc_balance >= amount0);

    println!("Alice transfers 1000 USDC to predicate\n");

    println!("Alice balances = {:#?}", alice.get_balances().await);
    println!("Predicate balances = {:#?}", predicate.get_balances().await);

    let _res = fulfill_order(
        &bob,
        &predicate,
        alice.address(),
        usdc.asset_id,
        amount0,
        uni.asset_id,
        amount1,
    )
    .await
    .unwrap();

    // println!("res = {:#?}", res);

    println!("Bob transfers 200 UNI to predicate, thus closing the order\n");

    // let predicate_balance = get_balance(&provider, predicate.address(), usdc.asset_id).await;
    let bob_uni_balance = bob.get_asset_balance(&uni.asset_id).await.unwrap();
    let bob_usdc_balance = bob.get_asset_balance(&usdc.asset_id).await.unwrap();
    let alice_uni_balance = alice.get_asset_balance(&uni.asset_id).await.unwrap();

    // The predicate root's coin has been spent
    // assert_eq!(predicate_balance, 0);

    // Receiver has been paid `ask_amount`
    assert_eq!(alice_uni_balance, initial_alice_uni_balance + amount1);

    // Taker has sent `ask_amount` of the asked token and received `amount0` of the offered token in return
    assert_eq!(bob_uni_balance, initial_bob_uni_balance - amount1);
    assert_eq!(bob_usdc_balance, initial_bob_usdc_balance + amount0);

    println!("Alice balance 200 UNI");
    println!("Bob balance 1000 USDC\n\n");
}
