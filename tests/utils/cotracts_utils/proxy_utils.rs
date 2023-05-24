use fuels::prelude::{abigen, Contract, LoadConfiguration, TxParameters, WalletUnlocked};

abigen!(Contract(
    name = "ProxyContract",
    abi = "proxy-contract/out/debug/proxy-contract-abi.json"
));

pub mod proxy_abi_calls {}

pub async fn get_proxy_contract_instance(wallet: &WalletUnlocked) -> ProxyContract<WalletUnlocked> {
    let path = "proxy-contract/out/debug/proxy-contract.bin";
    let id = Contract::load_from(path, LoadConfiguration::default())
        .unwrap()
        .deploy(wallet, TxParameters::default())
        .await
        .unwrap();
    let instance = ProxyContract::new(id, wallet.clone());
    instance
}
