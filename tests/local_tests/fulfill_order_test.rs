use fuels::accounts::predicate::Predicate;
use fuels::prelude::ViewOnlyAccount;
use fuels::types::{Address, Bits256};
use spark_sdk::{
    limit_orders_utils::{
        limit_orders_interactions::{create_order, fulfill_order},
        LimitOrderPredicateConfigurables,
    },
    proxy_utils::{deploy_proxy_contract, ProxySendFundsToPredicateParams},
};

use crate::utils::cotracts_utils::token_utils::{deploy_token_contract, Asset};
use crate::utils::local_tests_utils::init_wallets;
use crate::utils::print_title;

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
    let wallets = init_wallets().await;
    let admin = &wallets[0];
    let alice = &wallets[1];
    let alice_address = Address::from(alice.address());
    let bob = wallets[2].clone();
    let bob_address = Address::from(bob.address());

    println!("admin_address = 0x{:?}", Address::from(admin.address()));
    println!("alice_address = 0x{:?}", alice_address);
    println!("bob_address = 0x{:?}\n", bob_address);

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
    btc.mint(bob_address, amount1).await.unwrap();

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

    let proxy = deploy_proxy_contract(&alice, "proxy-contract/out/debug/proxy-contract.bin").await;
    let proxy_address = format!("0x{}", proxy.contract_id().hash);
    println!("proxyAddress = {:?}", proxy_address);
    create_order(&alice, &proxy_address, params, amount0)
        .await
        .unwrap();

    let initial_bob_usdc_balance = bob.get_asset_balance(&usdc.asset_id).await.unwrap();
    let initial_bob_btc_balance = bob.get_asset_balance(&btc.asset_id).await.unwrap();
    let initial_alice_btc_balance = alice.get_asset_balance(&btc.asset_id).await.unwrap();

    // The predicate root has received the coin
    let predicate_usdc_balance = predicate.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(predicate_usdc_balance, amount0);

    println!("Alice transfers 1000 USDC to predicate\n");

    fulfill_order(
        &bob,
        &predicate,
        alice.address(),
        usdc.asset_id,
        amount0,
        btc.asset_id,
        amount1,
    )
    .await
    .unwrap();

    println!("Bob transfers 200 BTC to predicate, thus closing the order\n");

    let predicate_balance = predicate.get_asset_balance(&usdc.asset_id).await.unwrap();
    let bob_btc_balance = bob.get_asset_balance(&btc.asset_id).await.unwrap();
    let bob_usdc_balance = bob.get_asset_balance(&usdc.asset_id).await.unwrap();
    let alice_btc_balance = alice.get_asset_balance(&btc.asset_id).await.unwrap();

    // The predicate root's coin has been spent
    assert_eq!(predicate_balance, 0);

    // Receiver has been paid `ask_amount`
    assert_eq!(alice_btc_balance, initial_alice_btc_balance + amount1);

    // Taker has sent `ask_amount` of the asked token and received `amount0` of the offered token in return
    assert_eq!(bob_btc_balance, initial_bob_btc_balance - amount1);
    assert_eq!(bob_usdc_balance, initial_bob_usdc_balance + amount0);

    println!("Alice balance 200 BTC");
    println!("Bob balance 1000 USDC\n\n");
}
