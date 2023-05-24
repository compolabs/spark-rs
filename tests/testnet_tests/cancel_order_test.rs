use std::{collections::HashMap, env, fs::read_to_string, str::FromStr};

use crate::utils::{
    cotracts_utils::{
        limit_orders_utils::{
            limit_orders_interactions::cancel_order, LimitOrdersPredicateConfigurables,
        },
        proxy_utils::{ProxyContract, ProxySendFundsToPredicateParams},
    },
    get_balance, print_title,
};
use dotenv::dotenv;
use fuels::{
    accounts::predicate::Predicate,
    prelude::{
        Bech32ContractId, CallParameters, Provider, TxParameters, ViewOnlyAccount, WalletUnlocked,
    },
    types::{Address, AssetId, Bits256, ContractId},
};
use serde::Deserialize;
use serde_json::from_str;

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
}

const RPC: &str = "beta-3.fuel.network";
const PROXY_ADDRESS: &str = "0x6f4b6994521d92664ec272382d86d3a3800a682fc53360854a9fc469e6ecd748";

#[tokio::test]
async fn cancel_order_test() {
    print_title("Cancel Order Test");
    dotenv().ok();

    //--------------- WALLETS ---------------
    let provider = Provider::connect(RPC).await.unwrap();
    let alice_pk = env::var("ALICE").unwrap().parse().unwrap();
    let alice = WalletUnlocked::new_from_private_key(alice_pk, Some(provider.clone()));
    let alice_address = Address::from(alice.address());

    println!("alice_address = 0x{:?}\n", alice_address);
    //--------------- TOKENS ---------------
    let token_configs: Vec<TokenConfig> =
        from_str(&read_to_string("tests/artefacts/tokens.json").unwrap()).unwrap();
    let mut tokens: HashMap<String, Token> = HashMap::new();
    for config in token_configs {
        tokens.insert(
            config.symbol.clone(),
            Token {
                decimals: config.decimals,
                asset_id: AssetId::from_str(&config.asset_id).unwrap(),
                contract_id: ContractId::from_str(&config.asset_id).unwrap(),
            },
        );
    }
    let usdc = tokens.get("USDC").unwrap();
    let uni = tokens.get("UNI").unwrap();

    let amount0 = 1000_000_000_u64; //1000 USDC
    let amount1 = 200_000_000_000_u64; //200 UNI
    println!("USDC AssetId (asset0) = 0x{:?}", usdc.asset_id);
    println!("UNI AssetId (asset1) = 0x{:?}", uni.asset_id);
    println!("amount0 = {:?} USDC", amount0 / 1000_000);
    println!("amount1 = {:?} UNI", amount1 / 1000_000_000);

    let price_decimals = 9;
    let exp = (price_decimals + usdc.decimals - uni.decimals).into();
    let price = amount1 * 10u64.pow(exp) / amount0;
    println!("Price = {:?}UNI/USDC\n", price);

    let initial_alice_usdc_balance = get_balance(&provider, alice.address(), usdc.asset_id).await;
    println!("Alice minting {:?} USDC\n", amount0 / 1000_000);

    //--------------- PREDICATE ---------

    let configurables = LimitOrdersPredicateConfigurables::new()
        .set_ASSET0(Bits256::from_hex_str(&usdc.asset_id.to_string()).unwrap())
        .set_ASSET1(Bits256::from_hex_str(&uni.asset_id.to_string()).unwrap())
        .set_ASSET0_DECINALS(usdc.decimals)
        .set_ASSET1_DECINALS(uni.decimals)
        .set_MAKER(Bits256::from_hex_str(&alice.address().hash().to_string()).unwrap())
        .set_PRICE(price);

    let predicate: Predicate = Predicate::load_from("./out/debug/limit-order-predicate.bin")
        .unwrap()
        .with_configurables(configurables);
    println!("Predicate root = {:?}\n", predicate.address());
    //--------------- THE TEST ---------
    // create_order(&alice, &predicate, &usdc_instance, amount0)
    //     .await
    //     .unwrap();
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
    // let proxy = get_proxy_contract_instance(&alice).await;
    let call_params = CallParameters::default()
        .set_asset_id(usdc.asset_id)
        .set_amount(amount0);
    let contract_id: Bech32ContractId = ContractId::from_str(&PROXY_ADDRESS).unwrap().into();
    let proxy = ProxyContract::new(contract_id, alice.clone());
    proxy
        .methods()
        .send_funds_to_predicate_root(params)
        .append_variable_outputs(1)
        .tx_params(TxParameters::default().set_gas_price(1))
        .call_params(call_params)
        .unwrap()
        .call()
        .await
        .unwrap();
    println!("Alice transfers 1000 USDC to predicate\n");

    cancel_order(&predicate, &alice, usdc.asset_id, amount0)
        .await
        .unwrap();

    println!("Alice canceles the order\n");
    // The predicate root's coin has been spent
    let predicate_balance = get_balance(&provider, predicate.address(), usdc.asset_id).await;
    assert_eq!(predicate_balance, 0);

    // Wallet balance is the same as before it sent the coins to the predicate
    let wallet_balance = get_balance(&provider, alice.address(), usdc.asset_id).await;
    assert_eq!(wallet_balance, initial_alice_usdc_balance);
    println!("Alice balance 1000 UDSC\n");
}
