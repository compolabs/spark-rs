use fuels::test_helpers::{launch_custom_provider_and_get_wallets, WalletsConfig};
use fuels::{prelude::ViewOnlyAccount, types::Address};
use spark_sdk::spark_utils::Spark;
use spark_sdk::print_title;
use src20_sdk::token_utils::{deploy_token_contract, Asset};

// example of inputs and outputs
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
    let config = WalletsConfig::new(Some(5), Some(1), Some(1_000_000_000));
    let wallets = launch_custom_provider_and_get_wallets(config, None, None)
        .await
        .unwrap();
    let admin = &wallets[0];
    let alice = &wallets[1];
    let alice_address = Address::from(alice.address());

    println!("alice_address = 0x{:?}\n", alice_address);
    //--------------- TOKENS ---------------
    let token_contract = deploy_token_contract(&admin).await;
    let usdc = Asset::new(admin.clone(), token_contract.contract_id().into(), "USDC");
    let btc = Asset::new(admin.clone(), token_contract.contract_id().into(), "BTC");

    let quote_amount = usdc.parse_units(40_000_f64) as u64; //40k USDC
    let base_amount = btc.parse_units(1_f64) as u64; // 1 BTC

    let price_decimals = 9;

    let exp = price_decimals + btc.decimals - usdc.decimals;
    let price = (quote_amount as u128 * 10u128.pow(exp as u32) / base_amount as u128) as u64;

    usdc.mint(alice_address, quote_amount).await.unwrap();
    let initial_alice_usdc_balance = alice.get_asset_balance(&usdc.asset_id).await.unwrap();

    let spark = Spark::deploy_proxy(admin, &btc, &usdc).await;
    let buy_predicate = spark.get_buy_predicate(alice, &btc, &usdc, price, 1);
    assert!(alice.get_asset_balance(&usdc.asset_id).await.unwrap() == quote_amount);

    spark
        .with_account(alice)
        .create_order(
            buy_predicate.address().into(),
            usdc.asset_id,
            quote_amount,
            price,
        )
        .await
        .unwrap();

    spark
        .cancel_order(&alice, &buy_predicate, usdc.asset_id, quote_amount)
        .await
        .unwrap();
    let predicate_balance = buy_predicate
        .get_asset_balance(&usdc.asset_id)
        .await
        .unwrap();
    assert_eq!(predicate_balance, 0);

    // Wallet balance is the same as before it sent the coins to the predicate
    let wallet_balance = alice.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(wallet_balance, initial_alice_usdc_balance);
}

/*

inputs = [ResourcePredicate { resource: Coin(Coin { amount: 1000000000, block_created: 8, asset_id: b89acd8db2a3c488cc7f802227f6bc4c3aa220f002803008cad93c7acb964f27, utxo_id: UtxoId { tx_id: bafa4bbe7f2222d99a363557721b36ab60217b6b91f76622dabbbdc342be0a38, output_index: 2 }, maturity: 0, owner: Bech32Address { hrp: "fuel", hash: 49a84788350c5ecf4a2135b2550c2d033742583e53e4d7c69a502ec404d47c1c }, status: Unspent }), code: [], data: UnresolvedBytes { data: [] } }]
outputs = [Coin { to: 5d99ee966b42cd8fc7bdd1364b389153a9e78b42b7d4a691470674e817888d4e, amount: 0, asset_id: b89acd8db2a3c488cc7f802227f6bc4c3aa220f002803008cad93c7acb964f27 }, Change { to: 5d99ee966b42cd8fc7bdd1364b389153a9e78b42b7d4a691470674e817888d4e, amount: 0, asset_id: b89acd8db2a3c488cc7f802227f6bc4c3aa220f002803008cad93c7acb964f27 }]

*/
