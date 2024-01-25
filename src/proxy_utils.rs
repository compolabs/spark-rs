use fuels::programs::call_utils::TxDependencyExtension;
use fuels::{
    accounts::wallet::WalletUnlocked,
    prelude::{abigen, Bech32ContractId, Contract, LoadConfiguration, TxPolicies},
    types::ContractId,
};
use std::str::FromStr;

abigen!(Contract(
    name = "ProxyContract",
    abi = "proxy-contract/out/debug/proxy-contract-abi.json"
));

pub async fn deploy_proxy_contract(
    wallet: &WalletUnlocked,
    bin_path: &str,
) -> ProxyContract<WalletUnlocked> {
    // let bin_path = "proxy-contract/out/debug/proxy-contract.bin";
    let id = Contract::load_from(bin_path, LoadConfiguration::default())
        .unwrap()
        .deploy(wallet, TxPolicies::default())
        .await
        .unwrap();
    ProxyContract::new(id, wallet.clone())
}

pub fn proxy_instance_by_address(
    wallet: &WalletUnlocked,
    address: &str,
) -> ProxyContract<WalletUnlocked> {
    let contract_id: Bech32ContractId = ContractId::from_str(address).unwrap().into();
    ProxyContract::new(contract_id, wallet.clone())
}

pub mod proxy_abi_calls {

    use fuels::{
        prelude::CallParameters, programs::call_response::FuelCallResponse, types::AssetId,
    };

    use super::*;

    pub async fn send_funds_to_predicate_root(
        instance: &ProxyContract<WalletUnlocked>,
        params: ProxySendFundsToPredicateParams,
        amount: u64,
    ) -> Result<FuelCallResponse<()>, fuels::types::errors::Error> {
        let call_params = CallParameters::default()
            .with_asset_id(AssetId::from(params.asset_0))
            .with_amount(amount);
        instance
            .methods()
            .send_funds_to_predicate_root(params)
            .append_variable_outputs(1)
            .with_tx_policies(TxPolicies::default().with_gas_price(1))
            .call_params(call_params)
            .unwrap()
            .call()
            .await
    }
}
