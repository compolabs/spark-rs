use crate::utils::{
    cotracts_utils::token_utils::{deploy_token_contract, Asset},
    local_tests_utils::init_wallets,
    print_title,
};
use fuels::{
    accounts::predicate::Predicate,
    prelude::ViewOnlyAccount,
    types::{Address, Bits256},
};
use spark_sdk::{
    limit_orders_utils::{
        limit_orders_interactions::{cancel_order, create_order},
        LimitOrderPredicateConfigurables,
    },
    proxy_utils::{deploy_proxy_contract, ProxySendFundsToPredicateParams},
};
// Alice wants to exchange 1000 USDC for 200 UNI
// Alice canceled order
/*
inputs
   ResourcePredicate { resource: Coin { amount: 1000000000, asset_id: USDC, owner: Predicate, status: Unspent }}
outputs
   Coin { to: Alice, amount: 0, asset_id: USDC }
   Change { to: Alice, amount: 0, asset_id: USDC }
 */
#[tokio::test]
async fn cancel_order_test() {
    print_title("Cancel Order Test");
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
    let price = amount1 * 10u64.pow(exp as u32) / amount0;
    println!("Price = {:?} BTC/USDC", price);

    usdc.mint(alice_address, amount0).await.unwrap();
    let initial_alice_usdc_balance = alice.get_asset_balance(&usdc.asset_id).await.unwrap();
    // btc.mint(bob_address, amount1).await.unwrap();

    println!("Alice minting {:?} USDC", amount0 / 1_000_000);
    println!("Bob minting {:?} BTC\n", amount1 / 1_00_000_000);

    //--------------- PREDICATE ---------

    let configurables = LimitOrderPredicateConfigurables::new()
        .with_ASSET0(usdc.asset_id.into())
        .with_ASSET1(btc.asset_id.into())
        .with_ASSET0_DECIMALS(usdc.decimals as u8)
        .with_ASSET1_DECIMALS(btc.decimals as u8)
        .with_MAKER(Bits256::from_hex_str(&alice.address().hash().to_string()).unwrap())
        .with_PRICE(price)
        .with_MIN_FULFILL_AMOUNT0(amount0);

    let predicate: Predicate =
        Predicate::load_from("./limit-order-predicate/out/debug/limit-order-predicate.bin")
            .unwrap()
            .with_configurables(configurables)
            .with_provider(admin.provider().unwrap().clone());
    println!("Predicate root = {:?}\n", predicate.address());

    // ==================== ALICE CREATES THE ORDER (TRANSFER) ====================
    // Alice transfer amount0 of  usdc.asset_id to the predicate root
    assert!(alice.get_asset_balance(&usdc.asset_id).await.unwrap() == amount0);
    let params = ProxySendFundsToPredicateParams {
        predicate_root: predicate.address().into(),
        asset_0: usdc.asset_id.into(),
        asset_1: btc.asset_id.into(),
        maker: alice_address,
        min_fulfill_amount_0: 1,
        price,
        asset_0_decimals: 6,
        asset_1_decimals: 9,
        price_decimals: 9,
    };

    let proxy = deploy_proxy_contract(alice, "proxy-contract/out/debug/proxy-contract.bin").await;
    let proxy_address = format!("0x{}", proxy.contract_id().hash);
    println!("proxyAddress = {:?}", proxy_address);
    println!("alice balance = {:#?}", alice.get_balances().await.unwrap());
    println!(
        "predicate balance = {:#?}",
        predicate.get_balances().await.unwrap()
    );
    create_order(&alice, &proxy_address, params, amount0)
        .await
        .unwrap();
    println!("alice balance = {:#?}", alice.get_balances().await.unwrap());
    println!(
        "predicate balance = {:#?}",
        predicate.get_balances().await.unwrap()
    );

    println!("Alice transfers 1000 USDC to predicate\n");
    // alice
    //     .clone()
    //     .transfer(
    //         predicate.address(),
    //         100,
    //         AssetId::BASE,
    //         TxPolicies::default(),
    //     )
    //     .await
    //     .unwrap();

    cancel_order(&alice, &predicate, usdc.asset_id, amount0)
        .await
        .unwrap();
    println!("alice balance = {:#?}", alice.get_balances().await.unwrap());
    println!(
        "predicate balance = {:#?}",
        predicate.get_balances().await.unwrap()
    );
    println!("Alice canceles the order\n");
    // The predicate root's coin has been spent
    let predicate_balance = predicate.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(predicate_balance, 0);

    // Wallet balance is the same as before it sent the coins to the predicate
    let wallet_balance = alice.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(wallet_balance, initial_alice_usdc_balance);
    println!("Alice balance 1000 UDSC\n");
}

/*

inputs = [ResourcePredicate { resource: Coin(Coin { amount: 1000000000, block_created: 8, asset_id: b89acd8db2a3c488cc7f802227f6bc4c3aa220f002803008cad93c7acb964f27, utxo_id: UtxoId { tx_id: bafa4bbe7f2222d99a363557721b36ab60217b6b91f76622dabbbdc342be0a38, output_index: 2 }, maturity: 0, owner: Bech32Address { hrp: "fuel", hash: 49a84788350c5ecf4a2135b2550c2d033742583e53e4d7c69a502ec404d47c1c }, status: Unspent }), code: [], data: UnresolvedBytes { data: [] } }]
outputs = [Coin { to: 5d99ee966b42cd8fc7bdd1364b389153a9e78b42b7d4a691470674e817888d4e, amount: 0, asset_id: b89acd8db2a3c488cc7f802227f6bc4c3aa220f002803008cad93c7acb964f27 }, Change { to: 5d99ee966b42cd8fc7bdd1364b389153a9e78b42b7d4a691470674e817888d4e, amount: 0, asset_id: b89acd8db2a3c488cc7f802227f6bc4c3aa220f002803008cad93c7acb964f27 }]

*/
