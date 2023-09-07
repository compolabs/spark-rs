use std::{collections::HashMap, env, fs::read_to_string, str::FromStr};

use dotenv::dotenv;
use fuels::{
    accounts::{predicate::Predicate, wallet::WalletUnlocked},
    prelude::{Bech32ContractId, Provider, ViewOnlyAccount},
    types::{Address, AssetId, Bits256, ContractId},
};
use serde::Deserialize;
use serde_json::from_str;
use spark_sdk::{
    limit_orders_utils::{
        limit_orders_interactions::{cancel_order, create_order},
        LimitOrderPredicateConfigurables,
    },
    proxy_utils::ProxySendFundsToPredicateParams,
};

use crate::utils::print_title;

#[derive(Deserialize)]
pub struct TokenConfig {
    symbol: String,
    decimals: u8,
    asset_id: String,
}
pub struct Token {
    decimals: u8,
    asset_id: AssetId,
    contract_id: ContractId,
    instance: TokenContract<WalletUnlocked>,
}

const RPC: &str = "beta-4.fuel.network";
const PROXY_ADDRESS: &str = "0x8924a38ac11879670de1d0898c373beb1e35dca974c4cab8a70819322f6bd9c4";
//https://spark-indexer.spark-defi.com/api/graph/compolabs/spark_indexer

#[tokio::test]
async fn cancel_order_test() {
    print_title("Cancel Order Test");
    dotenv().ok();

    //--------------- WALLETS ---------------
    let provider = Provider::connect(RPC).await.unwrap();

    let admin_pk = env::var("ADMIN").unwrap().parse().unwrap();
    let admin = WalletUnlocked::new_from_private_key(admin_pk, Some(provider.clone()));

    let alice_pk = env::var("ALICE").unwrap().parse().unwrap();
    let alice = WalletUnlocked::new_from_private_key(alice_pk, Some(provider.clone()));
    let alice_address = Address::from(alice.address());

    println!("alice_address = 0x{:?}\n", alice_address);
    //--------------- TOKENS ---------------
    let token_configs: Vec<TokenConfig> =
        from_str(&read_to_string("tests/artefacts/tokens.json").unwrap()).unwrap();
    let mut tokens: HashMap<String, Token> = HashMap::new();
    for config in token_configs {
        let contract_id: Bech32ContractId = ContractId::from_str(&config.asset_id).unwrap().into();
        tokens.insert(
            config.symbol.clone(),
            Token {
                instance: TokenContract::new(contract_id, admin.clone()),
                decimals: config.decimals,
                asset_id: AssetId::from_str(&config.asset_id).unwrap(),
                contract_id: ContractId::from_str(&config.asset_id).unwrap(),
            },
        );
    }
    let usdc = tokens.get("USDC").unwrap();
    let uni = tokens.get("UNI").unwrap();

    let amount0 = 1000_000_000_u64; //1000 USDC
    let amount1 = 300_000_000_000_u64; //200 UNI
    println!("USDC AssetId (asset0) = 0x{:?}", usdc.asset_id);
    println!("UNI AssetId (asset1) = 0x{:?}", uni.asset_id);
    println!("amount0 = {:?} USDC", amount0 / 1000_000);
    println!("amount1 = {:?} UNI", amount1 / 1000_000_000);

    let price_decimals = 9;
    let exp = (price_decimals + usdc.decimals - uni.decimals).into();
    let price = amount1 * 10u64.pow(exp) / amount0;
    println!("Price = {:?}UNI/USDC\n", price);
    let initial_alice_usdc_balance = alice.get_asset_balance(&usdc.asset_id).await.unwrap();
    if initial_alice_usdc_balance < amount0 {
        token_abi_calls::mint(&usdc.instance, amount0, alice_address)
            .await
            .unwrap();
        println!("Alice minting {:?} USDC\n", amount0 / 1000_000);
    }
    let initial_alice_usdc_balance = alice.get_asset_balance(&usdc.asset_id).await.unwrap();

    //--------------- PREDICATE ---------

    let configurables = LimitOrderPredicateConfigurables::new()
        .set_ASSET0(Bits256::from_hex_str(&usdc.asset_id.to_string()).unwrap())
        .set_ASSET1(Bits256::from_hex_str(&uni.asset_id.to_string()).unwrap())
        .set_ASSET0_DECIMALS(usdc.decimals)
        .set_ASSET1_DECIMALS(uni.decimals)
        .set_MAKER(Bits256::from_hex_str(&alice.address().hash().to_string()).unwrap())
        .set_PRICE(price);

    let predicate: Predicate =
        Predicate::load_from("./limit-order-predicate/out/debug/limit-order-predicate.bin")
            .unwrap()
            .with_configurables(configurables)
            .with_provider(admin.provider().unwrap().clone());
    println!("Predicate root = {:?}\n", predicate.address());
    //--------------- THE TEST ---------
    let params = ProxySendFundsToPredicateParams {
        predicate_root: predicate.address().into(),
        asset_0: usdc.contract_id.into(),
        asset_1: uni.contract_id.into(),
        maker: alice_address,
        min_fulfill_amount_0: 1,
        price,
        asset_0_decimals: 6,
        asset_1_decimals: 9,
        price_decimals: 9,
    };

    create_order(&alice, PROXY_ADDRESS, params, amount0)
        .await
        .unwrap();

    println!("Alice transfers 1000 USDC to predicate\n");

    cancel_order(&alice, &predicate, usdc.asset_id, amount0)
        .await
        .unwrap();

    println!("Alice canceles the order\n");
    // The predicate root's coin has been spent
    let predicate_balance = predicate.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(predicate_balance, 0);

    // Wallet balance is the same as before it sent the coins to the predicate
    let wallet_balance = alice.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(wallet_balance, initial_alice_usdc_balance);
    println!("Alice balance 1000 UDSC\n");
}
