use fuels::{
    accounts::predicate::Predicate,
    prelude::ViewOnlyAccount,
    types::{Address, Bits256},
};

use crate::utils::{
    cotracts_utils::{
        limit_orders_utils::{
            limit_orders_interactions::create_order, LimitOrdersPredicateConfigurables,
        },
        token_utils::{token_abi_calls, TokenContract},
    },
    local_tests_utils::{init_tokens, init_wallets},
    print_title,
};
// Alice wants to exchange 1000 USDC for 200 UNI
// Bob wants to exchange 200 UNI for 1000 USDC
/*
inputs
   ResourcePredicate { resource: Coin { amount: 1000000000, asset_id: USDC, owner: Predicate, status: Unspent }}
outputs
   Coin { to: Alice, amount: 0, asset_id: USDC }
   Change { to: Alice, amount: 0, asset_id: USDC }
 */
#[tokio::test]
async fn recreate_order_test() {
    print_title("Recreate Order Test");
    //--------------- WALLETS ---------------
    let wallets = init_wallets().await;
    let admin = &wallets[0];
    let alice = &wallets[1];
    let alice_address = Address::from(alice.address());
    // let provider = alice.provider().unwrap();

    println!("alice_address = 0x{:?}", alice_address);
    println!("");
    //--------------- TOKENS ---------------
    let assets = init_tokens(&admin).await;
    let usdc = assets.get("USDC").unwrap();
    let usdc_instance = TokenContract::new(usdc.contract_id.into(), admin.clone());
    let uni = assets.get("UNI").unwrap();

    let amount0 = 1000_000_000_u64; //1000 USDC
    let amount1 = 200_000_000_000_u64; //200 UNI
    println!("USDC AssetId (asset0) = {:?}", usdc.asset_id.to_string());
    println!("UNI AssetId (asset1) = {:?}", uni.asset_id.to_string());
    println!("amount0 = {:?} USDC", amount0 / 1000_000);
    println!("amount1 = {:?} UNI\n", amount1 / 1000_000_000);

    token_abi_calls::mint_and_transfer(&usdc_instance, amount0, alice_address).await;
    // let initial_alice_usdc_balance = get_balance(provider, alice.address(), usdc.asset_id).await;
    println!("Alice minting {:?} USDC\n", amount0 / 1000_000);

    //--------------- PREDICATE ---------
    //FIXME
    let exp = 1_000_000;
    let price = amount1 * exp / amount0;

    // let configurables = LimitOrdersPredicateConfigurables::new()
    //     .set_ASSET0(Bits256::from_hex_str(&usdc.asset_id.to_string()).unwrap())
    //     // .set_ASSET0_DECINALS(1u8)
    //     .set_ASSET1(Bits256::from_hex_str(&uni.asset_id.to_string()).unwrap())
    //     // .set_ASSET1_DECINALS(1u8)
    //     .set_MAKER(Bits256::from_hex_str(&alice.address().hash().to_string()).unwrap())
    //     .set_PRICE(price);

    let predicate: Predicate =
        Predicate::load_from("./out/debug/limit-order-predicate.bin").unwrap();
    // .with_configurables(configurables);
    println!("Predicate root = {:?}\n", predicate.address());
    //--------------- THE TEST ---------
    assert!(alice.get_asset_balance(&usdc.asset_id).await.unwrap() == amount0);
    create_order(&alice, &predicate, &usdc_instance, amount0)
        .await
        .unwrap();
    println!("Alice transfers 1000 USDC to predicate\n");

    // limit_orders_interactions::recreate_order(&predicate, &alice, usdc.asset_id, amount0)
    //     .await
    //     .unwrap();

    // println!("Alice recreatees the order\n");
    // // The predicate root's coin has been spent
    // let predicate_balance = get_balance(provider, predicate.address(), usdc.asset_id).await;
    // assert_eq!(predicate_balance, 0);

    // // Wallet balance is the same as before it sent the coins to the predicate
    // let wallet_balance = get_balance(provider, alice.address(), usdc.asset_id).await;
    // assert_eq!(wallet_balance, initial_alice_usdc_balance);
    // println!("Alice balance 1000 UDSC\n");
}
