contract;
use std::logging::log;
use std::call_frames::msg_asset_id;
use std::context::msg_amount;
use std::asset::transfer_to_address;
use std::constants::ZERO_B256;

abi ProxyContract {
    #[payable]
    fn send_funds_to_predicate_root(params: ProxySendFundsToPredicateParams);
}

struct ProxySendFundsToPredicateParams {
    predicate_root: Address,
    asset0: AssetId,
    asset1: AssetId,
    maker: Address,
    min_fulfill_amount0: u64,
    price: u64,
    asset0_decimals: u8,
    asset1_decimals: u8,
    price_decimals: u8,
}

impl ProxyContract for Contract {
    #[payable]
    fn send_funds_to_predicate_root(params: ProxySendFundsToPredicateParams) {
        let amount = msg_amount();
        assert(params.predicate_root != Address::from(ZERO_B256) && params.maker != Address::from(ZERO_B256));
        assert(params.asset0.into() != ZERO_B256 && params.asset1.into() != ZERO_B256);
        assert(amount > 0 && msg_asset_id() == params.asset0);
        assert(params.min_fulfill_amount0 > 0 && params.price > 0);
        assert(params.asset0_decimals >= 0u8 && params.asset1_decimals >= 0u8 && params.price_decimals >= 0u8); //TODO add <= 9 check
        log(params);
        transfer_to_address(params.predicate_root, params.asset0, amount);
    }
}
