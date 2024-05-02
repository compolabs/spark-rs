use fuels::test_helpers::{launch_custom_provider_and_get_wallets, WalletsConfig};
use fuels::{accounts::predicate::Predicate, prelude::ViewOnlyAccount, types::Address};
use spark_sdk::limit_orders_utils::{
    LimitOrderPredicateConfigurables, Proxy, ProxyContractConfigurables,
};
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

    let amount0 = usdc.parse_units(40_000_f64) as u64; //40k USDC
    let amount1 = btc.parse_units(1_f64) as u64; // 1 BTC
    println!("USDC AssetId (asset0) = 0x{:?}", usdc.asset_id);
    println!("BTC AssetId (asset1) = 0x{:?}", btc.asset_id);
    println!("amount0 = {:?} USDC", usdc.format_units(amount0 as f64));
    println!("amount1 = {:?} BTC", btc.format_units(amount1 as f64));

    let price_decimals = 9;
    let exp = price_decimals + usdc.decimals - btc.decimals;
    let price = amount0 * 10u64.pow(exp as u32) / amount1;
    println!("Price = {:?} BTC/USDC", price / 1_000_000_000);

    usdc.mint(alice_address, amount0).await.unwrap();
    println!("Alice minting {:?} USDC", usdc.format_units(amount0 as f64));

    //--------------- PREDICATE ---------
    let proxy_configurables = ProxyContractConfigurables::default()
        .with_BASE_ASSET(btc.asset_id)
        .with_BASE_ASSET_DECIMALS(btc.decimals as u32)
        .with_QUOTE_ASSET(usdc.asset_id)
        .with_QUOTE_ASSET_DECIMALS(usdc.decimals as u32);
    let proxy = Proxy::deploy(admin, proxy_configurables).await;

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

    // create_order(alice, predicate.address(), usdc.asset_id, amount0)
    //     .await
    //     .unwrap();
    proxy
        .with_account(alice)
        .create_order(predicate.address().into(), usdc.asset_id, amount0, price)
        .await
        .unwrap();

    assert!(alice.get_asset_balance(&usdc.asset_id).await.unwrap() == 0);
    assert!(predicate.get_asset_balance(&usdc.asset_id).await.unwrap() == amount0);
}
