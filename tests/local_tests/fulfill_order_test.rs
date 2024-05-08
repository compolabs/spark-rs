use fuels::test_helpers::{launch_custom_provider_and_get_wallets, WalletsConfig};
use fuels::{prelude::ViewOnlyAccount, types::Address};
use spark_sdk::spark_utils::Spark;
use spark_sdk::print_title;
use src20_sdk::token_utils::{deploy_token_contract, Asset};

// example of inputs and outputs
// Alice wants to exchange 1000 USDC for 200 BTC
// Bob wants to exchange 200 BTC for 1000 USDC
/*
inputs
    ResourcePredicate { resource: Coin { amount: 1000000000, asset_id: USDC, owner: Predicate, status: Unspent }}
    ResourceSigned { resource: Coin { amount: 200000000000, asset_id: BTC, owner: Bob, status: Unspent }}
outputs
    Coin { to: Alice, amount: 200000000000, asset_id: BTC }
    Change { to: Bob, amount: 0, asset_id: BTC }
    Coin { to: Bob, amount: 1000000000, asset_id: USDC }
    Change { to: Predicate, amount: 0, asset_id: USDC }
 */
#[tokio::test]
async fn fulfill_order_test() {
    print_title("Fulfill Order Test");
    //--------------- WALLETS ---------------
    let config = WalletsConfig::new(Some(5), Some(1), Some(1_000_000_000));
    let wallets = launch_custom_provider_and_get_wallets(config, None, None)
        .await
        .unwrap();
    let admin = &wallets[0];
    let alice = &wallets[1];
    let alice_address = Address::from(alice.address());
    let bob = wallets[2].clone();
    let bob_address = Address::from(bob.address());

    //--------------- TOKENS ---------------
    let token_contract = deploy_token_contract(&admin).await;
    let usdc = Asset::new(admin.clone(), token_contract.contract_id().into(), "USDC");
    let btc = Asset::new(admin.clone(), token_contract.contract_id().into(), "BTC");

    let quote_amount = usdc.parse_units(40_000_f64) as u64; //40k USDC
    let base_amount = btc.parse_units(1_f64) as u64; // 1 BTC

    let price_decimals = 9;

    let exp = price_decimals + btc.decimals - usdc.decimals;
    let price = (quote_amount as u128 * 10u128.pow(exp as u32) / base_amount as u128) as u64;
    println!("price = {:?}", price as f64 / 1e9);

    usdc.mint(alice_address, quote_amount).await.unwrap();
    btc.mint(bob_address, base_amount).await.unwrap();

    let spark = Spark::deploy_proxy(admin, &btc, &usdc).await;
    let buy_predicate = spark.get_buy_predicate(alice, &btc, &usdc, price, 1);

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

    let initial_bob_usdc_balance = bob.get_asset_balance(&usdc.asset_id).await.unwrap();
    let initial_bob_btc_balance = bob.get_asset_balance(&btc.asset_id).await.unwrap();
    let initial_alice_btc_balance = alice.get_asset_balance(&btc.asset_id).await.unwrap();

    // The predicate root has received the coin
    let predicate_usdc_balance = buy_predicate
        .get_asset_balance(&usdc.asset_id)
        .await
        .unwrap();
    assert_eq!(predicate_usdc_balance, quote_amount);

    spark.fulfill_order(
        &bob,
        &buy_predicate,
        alice.address(),
        usdc.asset_id,
        quote_amount,
        btc.asset_id,
        base_amount,
    )
    .await
    .unwrap();

    let predicate_balance = buy_predicate
        .get_asset_balance(&usdc.asset_id)
        .await
        .unwrap();
    let bob_btc_balance = bob.get_asset_balance(&btc.asset_id).await.unwrap();
    let bob_usdc_balance = bob.get_asset_balance(&usdc.asset_id).await.unwrap();
    let alice_btc_balance = alice.get_asset_balance(&btc.asset_id).await.unwrap();

    // The predicate root's coin has been spent
    assert_eq!(predicate_balance, 0);

    // Receiver has been paid `ask_amount`
    assert_eq!(alice_btc_balance, initial_alice_btc_balance + base_amount);

    // Taker has sent `ask_amount` of the asked token and received `quote_amount` of the offered token in return
    assert_eq!(bob_btc_balance, initial_bob_btc_balance - base_amount);
    assert_eq!(bob_usdc_balance, initial_bob_usdc_balance + quote_amount);
}
