use fuels::accounts::predicate::Predicate;
use fuels::prelude::ViewOnlyAccount;
use fuels::types::{Address, Bits256};

use crate::utils::cotracts_utils::limit_orders_utils::limit_orders_interactions::{
    create_order, fulfill_order,
};
use crate::utils::cotracts_utils::limit_orders_utils::LimitOrdersPredicateConfigurables;
use crate::utils::cotracts_utils::token_utils::{token_abi_calls, TokenContract};
use crate::utils::{get_balance, local_tests_utils::*, print_title};

// Alice wants to exchange 1000 USDC for 200 UNI
// Bob wants to exchange 200 UNI for 1000 USDC

#[tokio::test]
async fn partial_fulfill_order_test() {
    print_title("Partial fulfill Order Test");
    print_title("Fulfill Order Test");
    //--------------- WALLETS ---------------
    let wallets = init_wallets().await;
    let admin = wallets[0].clone();
    let alice = wallets[1].clone();
    let alice_address = Address::from(alice.address());
    let bob = wallets[2].clone();
    let bob_address = Address::from(bob.address());
    let provider = alice.provider().unwrap();

    println!("admin_address = 0x{:?}", Address::from(admin.address()));
    println!("alice_address = 0x{:?}", alice_address);
    println!("bob_address = 0x{:?}\n", bob_address);

    //--------------- TOKENS ---------------
    let assets = init_tokens(&admin).await;
    let usdc = assets.get("USDC").unwrap();
    let usdc_instance = TokenContract::new(usdc.contract_id.into(), admin.clone());
    let uni = assets.get("UNI").unwrap();
    let uni_instance = TokenContract::new(uni.contract_id.into(), admin.clone());

    let amount0 = 1_000_000_000; //1000 USDC
    let amount1 = 200_000_000_000; // 200 UNI
    println!("USDC AssetId (asset0) = 0x{:?}", usdc.asset_id.to_string());
    println!("UNI AssetId (asset1) = 0x{:?}", uni.asset_id.to_string());
    println!("amount0 = {:?} USDC", amount0 / 1_000_000);
    println!("amount1 = {:?} UNI\n", amount1 / 1_000_000_000);

    token_abi_calls::mint_and_transfer(&usdc_instance, amount0, alice_address).await;
    token_abi_calls::mint_and_transfer(&uni_instance, amount1, bob_address).await;

    println!("Alice minting {:?} USDC", amount0 / 1_000_000);
    println!("Bob minting {:?} UNI\n", amount1 / 1_000_000_000);

    //--------------- PREDICATE ---------
    //FIXME
    let exp = 1_000_000;
    let price = amount1 * exp / amount0;

    let configurables = LimitOrdersPredicateConfigurables::new()
        .set_ASSET0(Bits256::from_hex_str(&usdc.asset_id.to_string()).unwrap())
        // .set_ASSET0_DECINALS(1u8)
        .set_ASSET1(Bits256::from_hex_str(&uni.asset_id.to_string()).unwrap())
        // .set_ASSET1_DECINALS(1u8)
        .set_MAKER(Bits256::from_hex_str(&alice.address().hash().to_string()).unwrap())
        .set_PRICE(price);

    let predicate: Predicate = Predicate::load_from("./out/debug/limit-order-predicate.bin")
        .unwrap()
        .with_configurables(configurables);

    // ==================== ALICE CREATES THE ORDER (TRANSFER) ====================
    // Alice transfer amount0 of  usdc.asset_id to the predicate root
    assert!(alice.get_asset_balance(&usdc.asset_id).await.unwrap() == amount0);
    create_order(&alice, &predicate, &usdc_instance, amount0)
        .await
        .unwrap();

    let initial_bob_usdc_balance = get_balance(provider, bob.address(), usdc.asset_id).await;
    let initial_bob_uni_balance = get_balance(provider, bob.address(), uni.asset_id).await;
    let initial_alice_uni_balance = get_balance(provider, alice.address(), uni.asset_id).await;

    // The predicate root has received the coin
    let predicate_usdc_balance = get_balance(provider, predicate.address(), usdc.asset_id).await;
    assert_eq!(predicate_usdc_balance, amount0);

    println!("Alice transfers 1000 USDC to base predicate\n");

    let _res = fulfill_order(
        &predicate,
        &bob,
        &alice.address(),
        usdc.asset_id,
        amount0 / 2,
        uni.asset_id,
        amount1 / 2,
    )
    .await
    .unwrap();
    // The predicate root has received the coin
    println!("Bob transfers 100 UNI to base predicate, thus closing the order\n");
    let predicate_usdc_balance = get_balance(provider, predicate.address(), usdc.asset_id).await;
    let predicate_uni_balance = get_balance(provider, predicate.address(), uni.asset_id).await;
    let bob_uni_balance = get_balance(provider, bob.address(), uni.asset_id).await;
    let bob_usdc_balance = get_balance(provider, bob.address(), usdc.asset_id).await;
    let alice_uni_balance = get_balance(provider, &alice.address(), uni.asset_id).await;
    let alice_usdc_balance = get_balance(provider, &alice.address(), usdc.asset_id).await;
    assert_eq!(predicate_usdc_balance, amount0 / 2);
    assert_eq!(predicate_uni_balance, 0);
    assert_eq!(alice_usdc_balance, 0);
    assert_eq!(bob_usdc_balance, amount0 / 2);
    assert_eq!(alice_uni_balance, amount1 / 2);
    assert_eq!(bob_uni_balance, amount1 / 2);

    let _res = fulfill_order(
        &predicate,
        &bob,
        alice.address(),
        usdc.asset_id,
        amount0 / 2,
        uni.asset_id,
        amount1 / 2,
    )
    .await
    .unwrap();
    println!("Bob transfers another 100 UNI to new predicate, thus closing the order\n");
    let predicate_usdc_balance = get_balance(provider, predicate.address(), usdc.asset_id).await;
    let bob_uni_balance = get_balance(provider, bob.address(), uni.asset_id).await;
    let bob_usdc_balance = get_balance(provider, bob.address(), usdc.asset_id).await;
    let alice_uni_balance = get_balance(provider, &alice.address(), uni.asset_id).await;

    // The predicate root's coin has been spent
    assert_eq!(predicate_usdc_balance, 0);

    // Receiver has been paid `ask_amount`
    assert_eq!(alice_uni_balance, initial_alice_uni_balance + amount1);

    // Taker has sent `ask_amount` of the asked token and received `amount0` of the offered token in return
    assert_eq!(bob_uni_balance, initial_bob_uni_balance - amount1);
    assert_eq!(bob_usdc_balance, initial_bob_usdc_balance + amount0);

    println!("Alice balance 200 UNI");
    println!("Bob balance 1000 USDC\n\n");
}