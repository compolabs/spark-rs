predicate;
mod utils;

use std::u128::U128;
use std::outputs::{Output, output_amount, output_count, output_type};
use std::inputs::{input_amount, input_asset_id, input_count, input_owner};
use std::constants::{ZERO_B256};
use utils::{output_coin_to, verify_output_coin};

configurable {
    ASSET0: b256 = ZERO_B256,
    ASSET1: b256 = ZERO_B256,
    MAKER: b256 = ZERO_B256,
    PRICE: u64 = 0,
    MIN_FULFILL_AMOUNT0: u64 = 1,
    ASSET0_DECINALS: u8 = 1,
    ASSET1_DECINALS: u8 = 1,
}

impl U128 {
    pub fn from_u64(value: u64) -> U128 {
        U128::from((0, value))
    }
}

fn main() -> bool {
    let mut i = 0u8;
    let inputs: u8 = input_count();
    while i < if inputs > 2u8 { 2u8 } else { inputs } {
        if input_owner(i).unwrap() == Address::from(MAKER) {
            return true;
        }
        i += 1u8;
    }
    let price_decimals = 9;
    let asset0_amount_u64 = output_amount(2);
    let asset1_amount_u64 = output_amount(0);
    let exp = U128::from_u64(price_decimals + ASSET0_DECINALS - ASSET1_DECINALS);
    let price = U128::from_u64(asset1_amount_u64) * U128::from_u64(10).pow(exp) / U128::from_u64(asset0_amount_u64);
    assert(price.as_u64().unwrap() >= PRICE);
    assert(PRICE > 0 && ASSET0 != ZERO_B256 && ASSET1 != ZERO_B256 && MAKER != ZERO_B256);
    assert(input_asset_id(0).unwrap().into() == ASSET0);

    let asset0_balance = input_amount(0).unwrap();
    let limit = if asset0_balance >= MIN_FULFILL_AMOUNT0 {
        MIN_FULFILL_AMOUNT0
    } else {
        asset0_balance
    };
    assert(asset0_amount_u64 >= limit);

    match output_type(0) {
        Output::Coin => (),
        _ => revert(0),
    };
    assert(verify_output_coin(0));
    assert(output_coin_to(0) == MAKER);
    true
}
