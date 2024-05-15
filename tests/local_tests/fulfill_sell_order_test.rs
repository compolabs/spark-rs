use fuels::test_helpers::{launch_custom_provider_and_get_wallets, WalletsConfig};
use fuels::{prelude::ViewOnlyAccount, types::Address};
use spark_sdk::print_title;
use spark_sdk::spark_utils::Spark;
use src20_sdk::token_utils::{deploy_token_contract, Asset};

#[tokio::test]
async fn fulfill_sell_order_test() {
    print_title("Fulfill Sell Order Test");
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

    btc.mint(alice_address, base_amount).await.unwrap();
    usdc.mint(bob_address, quote_amount).await.unwrap();

    //--------------- PREDICATE ---------
    let spark = Spark::deploy_proxy(admin, &btc, &usdc).await;
    let sell_predicate = spark.get_sell_predicate(alice, &btc, &usdc, price, 1);
    let root = sell_predicate.address();

    let initial_bob_btc_balance = bob.get_asset_balance(&btc.asset_id).await.unwrap();
    let initial_bob_usdc_balance = bob.get_asset_balance(&usdc.asset_id).await.unwrap();
    let initial_alice_btc_balance = alice.get_asset_balance(&btc.asset_id).await.unwrap();
    let initial_alice_usdc_balance = alice.get_asset_balance(&usdc.asset_id).await.unwrap();

    assert_eq!(initial_bob_btc_balance, 0);
    assert_eq!(initial_bob_usdc_balance, quote_amount);
    assert_eq!(initial_alice_btc_balance, base_amount);
    assert_eq!(initial_alice_usdc_balance, 0);

    spark
        .with_account(alice)
        .create_order(root.into(), btc.asset_id, base_amount, price)
        .await
        .unwrap();

    // The predicate root has received the coin
    let predicate_btc_balance = sell_predicate
        .get_asset_balance(&btc.asset_id)
        .await
        .unwrap();
    assert_eq!(predicate_btc_balance, base_amount);

    spark
        .fulfill_order(
            &bob,
            &sell_predicate,
            alice.address(),
            btc.asset_id,
            base_amount,
            usdc.asset_id,
            quote_amount,
        )
        .await
        .unwrap();

    let predicate_balance = sell_predicate
        .get_asset_balance(&usdc.asset_id)
        .await
        .unwrap();

    let bob_btc_balance = bob.get_asset_balance(&btc.asset_id).await.unwrap();
    let bob_usdc_balance = bob.get_asset_balance(&usdc.asset_id).await.unwrap();
    let alice_btc_balance = alice.get_asset_balance(&btc.asset_id).await.unwrap();
    let alice_usdc_balance = alice.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(bob_btc_balance, base_amount);
    assert_eq!(bob_usdc_balance, 0);
    assert_eq!(alice_btc_balance, 0);
    assert_eq!(alice_usdc_balance, quote_amount);
    assert_eq!(predicate_balance, 0);
}

#[tokio::test]
async fn partial_fulfill_sell_order_test() {
    print_title("Fulfill Sell Order Test");
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

    btc.mint(alice_address, base_amount).await.unwrap();
    usdc.mint(bob_address, quote_amount).await.unwrap();

    //--------------- PREDICATE ---------
    let spark = Spark::deploy_proxy(admin, &btc, &usdc).await;
    let sell_predicate = spark.get_sell_predicate(alice, &btc, &usdc, price, 1);
    let root = sell_predicate.address();

    let initial_bob_btc_balance = bob.get_asset_balance(&btc.asset_id).await.unwrap();
    let initial_bob_usdc_balance = bob.get_asset_balance(&usdc.asset_id).await.unwrap();
    let initial_alice_btc_balance = alice.get_asset_balance(&btc.asset_id).await.unwrap();
    let initial_alice_usdc_balance = alice.get_asset_balance(&usdc.asset_id).await.unwrap();

    assert_eq!(initial_bob_btc_balance, 0);
    assert_eq!(initial_bob_usdc_balance, quote_amount);
    assert_eq!(initial_alice_btc_balance, base_amount);
    assert_eq!(initial_alice_usdc_balance, 0);

    spark
        .with_account(alice)
        .create_order(root.into(), btc.asset_id, base_amount, price)
        .await
        .unwrap();

    // The predicate root has received the coin
    let predicate_btc_balance = sell_predicate
        .get_asset_balance(&btc.asset_id)
        .await
        .unwrap();
    assert_eq!(predicate_btc_balance, base_amount);

    spark
        .fulfill_order(
            &bob,
            &sell_predicate,
            alice.address(),
            btc.asset_id,
            base_amount * 3 / 4,
            usdc.asset_id,
            quote_amount * 3 / 4,
        )
        .await
        .unwrap();

    spark
        .fulfill_order(
            &bob,
            &sell_predicate,
            alice.address(),
            btc.asset_id,
            base_amount / 4,
            usdc.asset_id,
            quote_amount / 4,
        )
        .await
        .unwrap();

    let predicate_balance = sell_predicate
        .get_asset_balance(&usdc.asset_id)
        .await
        .unwrap();

    let bob_btc_balance = bob.get_asset_balance(&btc.asset_id).await.unwrap();
    let bob_usdc_balance = bob.get_asset_balance(&usdc.asset_id).await.unwrap();
    let alice_btc_balance = alice.get_asset_balance(&btc.asset_id).await.unwrap();
    let alice_usdc_balance = alice.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(bob_btc_balance, base_amount);
    assert_eq!(bob_usdc_balance, 0);
    assert_eq!(alice_btc_balance, 0);
    assert_eq!(alice_usdc_balance, quote_amount);
    assert_eq!(predicate_balance, 0);
}
