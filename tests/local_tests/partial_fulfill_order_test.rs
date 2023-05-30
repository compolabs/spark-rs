use fuels::accounts::predicate::Predicate;
use fuels::prelude::ViewOnlyAccount;
use fuels::types::{Address, Bits256};

use crate::utils::cotracts_utils::limit_orders_utils::limit_orders_interactions::{
    create_order, fulfill_order,
};
use crate::utils::cotracts_utils::limit_orders_utils::LimitOrderPredicateConfigurables;
use crate::utils::cotracts_utils::token_utils::{token_abi_calls, TokenContract};
use crate::utils::{get_balance, local_tests_utils::*, print_title};

// Alice wants to exchange 1000 USDC for 200 UNI
// Bob wants to exchange 200 UNI for 1000 USDC
/*
inputs
    ResourcePredicate { resource: Coin { amount: 1000000000, asset_id: USDC, utxo_id: UtxoId { output_index: 0 }, owner: Predicate, status: Unspent })}
    ResourceSigned { resource: Coin { amount: 200000000000, asset_id: UNI, utxo_id: UtxoId { output_index: 1 }, owner: Bob, status: Unspent })}]
outputs
    Coin { to: Alice, amount: 100000000000, asset_id: UNI }
    Change { to: Bob, amount: 0, asset_id: UNI }
    Coin { to: Bob, amount: 500000000, asset_id: USDC }
    Change { to: Predicate, amount: 0, asset_id: USDC }]

inputs
    ResourcePredicate { resource: Coin { amount: 500000000, asset_id: USDC, utxo_id: UtxoId { output_index: 3 }, owner: Predicate, status: Unspent })}
    ResourceSigned { resource: Coin { amount: 10000000000, asset_id: UNI, utxo_id: UtxoId { output_index: 1 }, owner: Bob, status: Unspent })}]
outputs
    Coin { to: Alice, amount: 100000000000, asset_id: UNI }
    Change { to: Bob, amount: 0, asset_id: UNI }
    Coin { to: Bob, amount: 500000000, asset_id: USDC }
    Change { to: Predicate, amount: 0, asset_id: USDC }]
*/
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
    println!("USDC AssetId (asset0) = 0x{:?}", usdc.asset_id);
    println!("UNI AssetId (asset1) = 0x{:?}", uni.asset_id);
    println!("amount0 = {:?} USDC", amount0 / 1_000_000);
    println!("amount1 = {:?} UNI", amount1 / 1_000_000_000);

    let price_decimals = 9;
    let exp = (price_decimals + usdc.config.decimals - uni.config.decimals).into();
    let price = amount1 * 10u64.pow(exp) / amount0;
    println!("Price = {:?}\n UNI/USDC", price);

    token_abi_calls::mint_and_transfer(&usdc_instance, amount0, alice_address).await;
    token_abi_calls::mint_and_transfer(&uni_instance, amount1, bob_address).await;

    println!("Alice minting {:?} USDC", amount0 / 1_000_000);
    println!("Bob minting {:?} UNI\n", amount1 / 1_000_000_000);

    //--------------- PREDICATE ---------
    //FIXME
    let exp = 1_000_000;
    let price = amount1 * exp / amount0;

    let configurables = LimitOrderPredicateConfigurables::new()
        .set_ASSET0(Bits256::from_hex_str(&usdc.asset_id.to_string()).unwrap())
        .set_ASSET1(Bits256::from_hex_str(&uni.asset_id.to_string()).unwrap())
        .set_MAKER(Bits256::from_hex_str(&alice.address().hash().to_string()).unwrap())
        .set_ASSET0_DECINALS(usdc.config.decimals)
        .set_ASSET1_DECINALS(uni.config.decimals)
        .set_PRICE(price)
        .set_MIN_FULFILL_AMOUNT0(amount0 / 2);

    let predicate: Predicate = Predicate::load_from("./limit-order-predicate/out/debug/limit-order-predicate.bin")
        .unwrap()
        .with_configurables(configurables);
    println!("Predicate root = {:?}\n", predicate.address());

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
        amount0 / 4 * 3,
        uni.asset_id,
        amount1 / 4 * 3,
    )
    .await
    .unwrap();
    // The predicate root has received the coin
    println!(
        "Bob transfers {} UNI to base predicate, thus closing the order\n",
        amount1 / 4 * 3 / 10u64.pow(9)
    );
    let predicate_usdc_balance = get_balance(provider, predicate.address(), usdc.asset_id).await;
    let predicate_uni_balance = get_balance(provider, predicate.address(), uni.asset_id).await;
    let bob_uni_balance = get_balance(provider, bob.address(), uni.asset_id).await;
    let bob_usdc_balance = get_balance(provider, bob.address(), usdc.asset_id).await;
    let alice_uni_balance = get_balance(provider, &alice.address(), uni.asset_id).await;
    let alice_usdc_balance = get_balance(provider, &alice.address(), usdc.asset_id).await;
    assert_eq!(predicate_usdc_balance, amount0 / 4);
    assert_eq!(predicate_uni_balance, 0);
    assert_eq!(alice_usdc_balance, 0);
    assert_eq!(bob_usdc_balance, amount0 / 4 * 3);
    assert_eq!(alice_uni_balance, amount1 / 4 * 3);
    assert_eq!(bob_uni_balance, amount1 / 4);

    let _res = fulfill_order(
        &predicate,
        &bob,
        alice.address(),
        usdc.asset_id,
        amount0 / 4,
        uni.asset_id,
        amount1 / 4,
    )
    .await
    .unwrap();
    println!(
        "Bob transfers another {} UNI to new predicate, thus closing the order\n",
        amount1 / 4 / 10u64.pow(9)
    );
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
