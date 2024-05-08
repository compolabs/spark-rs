use fuels::accounts::predicate::Predicate;
use fuels::accounts::wallet::WalletUnlocked;
use fuels::prelude::Account;
use fuels::prelude::Bech32Address;
use fuels::prelude::TxPolicies;
use fuels::programs::call_response::FuelCallResponse;
use fuels::programs::call_utils::TxDependencyExtension;
use fuels::programs::script_calls::ScriptCallHandler;
use fuels::types::unresolved_bytes::UnresolvedBytes;
use fuels::types::Address;
use fuels::types::AssetId;
use fuels::types::ContractId;
use fuels::{
    prelude::abigen,
    programs::contract::{CallParameters, Contract, LoadConfiguration},
};
use rand::Rng;
use src20_sdk::token_utils::Asset;
use std::path::PathBuf;
use std::str::FromStr;
abigen!(
    Predicate(
        name = "BuyPredicate",
        abi = "predicate-buy/out/debug/predicate-buy-abi.json"
    ),
    Predicate(
        name = "SellPredicate",
        abi = "predicate-sell/out/debug/predicate-sell-abi.json"
    ),
    Contract(
        name = "ProxyContract",
        abi = "proxy-contract/out/debug/proxy-contract-abi.json"
    )
);

const PROXY_BIN_PATH: &str = "proxy-contract/out/debug/proxy-contract.bin";
const PREDICATE_BUY_BIN_PATH: &str = "predicate-buy/out/debug/predicate-buy.bin";
const PREDICATE_SELL_BIN_PATH: &str = "predicate-sell/out/debug/predicate-sell.bin";

pub struct Spark {
    pub proxy: ProxyContract<WalletUnlocked>,
}

impl Spark {
    pub fn get_buy_predicate(
        &self,
        wallet: &WalletUnlocked,
        base_asset: &Asset,
        quote_asset: &Asset,
        price: u64,
        min_fulfill_quote_amount: u64,
    ) -> Predicate {
        let configurables = BuyPredicateConfigurables::new()
            .with_QUOTE_ASSET(quote_asset.asset_id.into())
            .with_BASE_ASSET(base_asset.asset_id.into())
            .with_QUOTE_DECIMALS(quote_asset.decimals as u32)
            .with_BASE_DECIMALS(base_asset.decimals as u32)
            .with_MAKER(wallet.address().into())
            .with_PRICE(price)
            .with_MIN_FULFILL_QUOTE_AMOUNT(min_fulfill_quote_amount);

        Predicate::load_from(PREDICATE_BUY_BIN_PATH)
            .unwrap()
            .with_configurables(configurables)
            .with_provider(wallet.provider().unwrap().clone())
    }
    pub fn get_sell_predicate(
        &self,
        wallet: &WalletUnlocked,
        base_asset: &Asset,
        quote_asset: &Asset,
        price: u64,
        min_fulfill_base_amount: u64,
    ) -> Predicate {
        let configurables = SellPredicateConfigurables::new()
            .with_QUOTE_ASSET(quote_asset.asset_id.into())
            .with_BASE_ASSET(base_asset.asset_id.into())
            .with_QUOTE_DECIMALS(quote_asset.decimals as u32)
            .with_BASE_DECIMALS(base_asset.decimals as u32)
            .with_MAKER(wallet.address().into())
            .with_PRICE(price)
            .with_MIN_FULFILL_BASE_AMOUNT(min_fulfill_base_amount);

        Predicate::load_from(PREDICATE_SELL_BIN_PATH)
            .unwrap()
            .with_configurables(configurables)
            .with_provider(wallet.provider().unwrap().clone())
    }

    pub async fn cancel_order(
        &self,
        wallet: &WalletUnlocked,
        predicate: &Predicate,
        asset0: AssetId,
        amount0: u64,
    ) -> Result<FuelCallResponse<()>, fuels::prelude::Error> {
        let provider = wallet.provider().unwrap();
        let mut predicate = predicate.clone();
        predicate.set_provider(provider.clone());

        let mut inputs = vec![];

        let mut inputs_predicate = predicate
            .get_asset_inputs_for_amount(asset0, amount0)
            .await
            .unwrap();
        inputs.append(&mut inputs_predicate);

        let mut outputs = vec![];
        let mut output_to_maker = wallet.get_asset_outputs_for_amount(wallet.address(), asset0, 0);
        outputs.append(&mut output_to_maker);
        // println!("inputs = {:?}", inputs);
        // println!("outputs = {:?}", outputs);
        let script_call = ScriptCallHandler::new(
            vec![],
            UnresolvedBytes::default(),
            wallet.clone(),
            provider.clone(),
            Default::default(),
        )
        .with_inputs(inputs)
        .with_outputs(outputs)
        .with_tx_policies(TxPolicies::default().with_gas_price(1));

        script_call.call().await
    }

    pub async fn fulfill_order(
        &self,
        wallet: &WalletUnlocked,
        predicate: &Predicate,
        maker_address: &Bech32Address,
        asset0: AssetId,
        amount0: u64,
        asset1: AssetId,
        amount1: u64,
    ) -> Result<FuelCallResponse<()>, fuels::prelude::Error> {
        let provider = wallet.provider().unwrap();
        let mut predicate = predicate.clone();
        predicate.set_provider(provider.clone());

        let mut inputs = vec![];
        // let balance = predicate.get_asset_balance(&asset0).await.unwrap_or(0);
        let mut inputs_predicate = predicate
            .get_asset_inputs_for_amount(asset0, 1)
            .await
            .unwrap();
        inputs.append(&mut inputs_predicate);
        let mut inputs_from_taker = wallet
            .get_asset_inputs_for_amount(asset1, amount1)
            .await
            .unwrap();
        inputs.append(&mut inputs_from_taker);

        // Output for the asked coin transferred from the taker to the receiver
        let mut outputs = vec![];
        let mut output_to_maker =
            wallet.get_asset_outputs_for_amount(maker_address, asset1, amount1);
        outputs.append(&mut output_to_maker);

        // Output for the offered coin transferred from the predicate to the order taker
        let mut output_to_taker =
            predicate.get_asset_outputs_for_amount(wallet.address(), asset0, amount0);
        outputs.append(&mut output_to_taker);

        // Change output for unspent asked asset
        // let output_asked_change =
        //     wallet.get_asset_outputs_for_amount(wallet.address(), asset1, 0)[1];
        // outputs.push(output_asked_change);

        // Partial fulfill output
        // let balance = predicate.get_asset_balance(&asset0).await.unwrap_or(0);
        // if balance > amount0 {
        //     let partial_fulfill_output = predicate.get_asset_outputs_for_amount(
        //         predicate.address(),
        //         asset0,
        //         balance - amount0,
        //     )[0];
        //     outputs.push(partial_fulfill_output);
        // }

        let script_call = ScriptCallHandler::new(
            vec![],
            UnresolvedBytes::default(),
            wallet.clone(),
            provider.clone(),
            Default::default(),
        )
        .with_inputs(inputs)
        .with_outputs(outputs)
        .with_tx_policies(TxPolicies::default().with_gas_price(1));

        script_call.call().await
    }

    pub async fn create_order(
        &self,
        predicate_root: Address,
        payment_asset: AssetId,
        payment_size: u64,
        base_price: u64,
    ) -> Result<FuelCallResponse<()>, fuels::types::errors::Error> {
        let call_params: CallParameters = CallParameters::default()
            .with_asset_id(payment_asset)
            .with_amount(payment_size);
        self.proxy
            .methods()
            .create_order(base_price, predicate_root, None)
            .append_variable_outputs(1)
            .call_params(call_params)
            .unwrap()
            .with_tx_policies(TxPolicies::default().with_gas_price(1))
            .call()
            .await
    }

    pub fn with_account(&self, account: &WalletUnlocked) -> Self {
        Self {
            proxy: self.proxy.with_account(account.clone()).unwrap(),
        }
    }

    pub async fn new(wallet: &WalletUnlocked, contract_id: &str) -> Self {
        let proxy = ProxyContract::new(
            &ContractId::from_str(contract_id).unwrap().into(),
            wallet.clone(),
        );
        Self { proxy }
    }

    pub async fn deploy_proxy(
        wallet: &WalletUnlocked,
        base_asset: &Asset,
        quote_asset: &Asset,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();

        let proxy_configurables = ProxyContractConfigurables::default()
            .with_BASE_ASSET(base_asset.asset_id)
            .with_BASE_ASSET_DECIMALS(base_asset.decimals as u32)
            .with_QUOTE_ASSET(quote_asset.asset_id)
            .with_QUOTE_ASSET_DECIMALS(quote_asset.decimals as u32);
        let config = LoadConfiguration::default().with_configurables(proxy_configurables);

        let bin_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(PROXY_BIN_PATH);
        let id = Contract::load_from(bin_path, config)
            .unwrap()
            .with_salt(salt)
            .deploy(wallet, TxPolicies::default().with_gas_price(1))
            .await
            .unwrap();

        let proxy = ProxyContract::new(id, wallet.clone());

        Self { proxy }
    }
}
