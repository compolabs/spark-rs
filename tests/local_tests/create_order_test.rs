use fuels::test_helpers::{launch_custom_provider_and_get_wallets, WalletsConfig};
use fuels::{prelude::ViewOnlyAccount, types::Address};
use spark_sdk::spark_utils::Spark;
use spark_sdk::print_title;
use src20_sdk::token_utils::{deploy_token_contract, Asset};

#[tokio::test]
async fn create_order_test() {
    print_title("Create Order Test");
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

    let spark = Spark::deploy_proxy(admin, &btc, &usdc).await;
    let buy_predicate = spark.get_buy_predicate(alice, &btc, &usdc, price, 1);
    assert!(alice.get_asset_balance(&usdc.asset_id).await.unwrap() == quote_amount);

    // create_order(alice, predicate.address(), usdc.asset_id, quote_amount)
    //     .await
    //     .unwrap();
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

    assert!(alice.get_asset_balance(&usdc.asset_id).await.unwrap() == 0);
    let predicate_balance = buy_predicate
        .get_asset_balance(&usdc.asset_id)
        .await
        .unwrap();
    assert!(predicate_balance == quote_amount);
}
