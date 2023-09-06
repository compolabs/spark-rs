use fuels::{
    accounts::wallet::WalletUnlocked,
    test_helpers::{launch_custom_provider_and_get_wallets, WalletsConfig},
};

pub async fn init_wallets() -> Vec<WalletUnlocked> {
    let config = WalletsConfig::new(Some(5), Some(1), Some(1_000_000_000));
    launch_custom_provider_and_get_wallets(config, None, None).await
}
