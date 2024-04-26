use fuels::prelude::abigen;

abigen!(Predicate(
    name = "LimitOrderPredicate",
    abi = "limit-order-predicate/out/debug/limit-order-predicate-abi.json"
));

pub mod limit_orders_interactions {

    use fuels::accounts::predicate::Predicate;
    use fuels::accounts::wallet::WalletUnlocked;
    use fuels::prelude::Account;
    use fuels::prelude::Bech32Address;
    use fuels::prelude::TxPolicies;
    use fuels::programs::call_response::FuelCallResponse;
    use fuels::programs::script_calls::ScriptCallHandler;
    use fuels::tx::Bytes32;
    use fuels::tx::Receipt;
    use fuels::types::unresolved_bytes::UnresolvedBytes;
    use fuels::types::AssetId;

    pub async fn cancel_order(
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
        wallet: &WalletUnlocked,
        predicate_root: &Bech32Address,
        asset_id: AssetId,
        amount: u64,
    ) -> Result<(Bytes32, Vec<Receipt>), fuels::prelude::Error> {
        let policies = TxPolicies::default().with_gas_price(1);
        wallet
            .transfer(predicate_root, amount, asset_id, policies)
            .await
    }
}
