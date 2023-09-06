predicate;
mod utils;

use std::u128::U128;
use std::outputs::{Output, output_amount, output_count, output_type};
use std::inputs::{input_amount, input_asset_id, input_count, input_owner};
use std::constants::{ZERO_B256};
use utils::{output_coin_to, verify_output_coin};

configurable {
    ASSET0: b256 = ZERO_B256, // Asset that provides maker(Alice)
    ASSET1: b256 = ZERO_B256, // Asset that provides taker(Bob)
    MAKER: b256 = ZERO_B256, // Order owner
    PRICE: u64 = 0, // asset1_amount / asset0_amount
    ASSET0_DECIMALS: u64 = 1,
    ASSET1_DECIMALS: u64 = 1,
    PRICE_DECIMALS: u64 = 9, // optional
    MIN_FULFILL_AMOUNT0: u64 = 1, // optional
}

impl U128 {
    pub fn from_u64(value: u64) -> U128 {
        U128::from((0, value))
    }
}

fn main() -> bool {
    let mut i = 0;
    let inputs = input_count();
    while i < inputs  {
        if input_owner(i).unwrap() == Address::from(MAKER) {
            return true;
        }
        i += 1;
    }
    let asset0_amount_u64 = output_amount(2);
    let asset1_amount_u64 = output_amount(0);
    let exp = U128::from_u64(PRICE_DECIMALS + ASSET0_DECIMALS - ASSET1_DECIMALS);
    let price = U128::from_u64(asset1_amount_u64) * U128::from_u64(10).pow(exp) / U128::from_u64(asset0_amount_u64);
    assert(price.as_u64().unwrap() >= PRICE);
    assert(PRICE > 0 && ASSET0 != ZERO_B256 && ASSET1 != ZERO_B256 && MAKER != ZERO_B256);
    assert(input_asset_id(0).unwrap() == ASSET0);

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
