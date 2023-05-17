predicate;
mod utils;

//use std::u128::U128;
use std::outputs::{Output, output_type};
use std::inputs::{input_count, input_owner};
use std::constants::ZERO_B256;
use utils::{
    input_coin_asset_id,
    output_coin_amount,
    output_coin_asset_id,
    output_coin_to,
    verify_output_coin,
};

configurable {
    ASSET0: b256 = ZERO_B256,
    ASSET1: b256 = ZERO_B256,
    MAKER: b256 = ZERO_B256,
    PRICE: u64 = 0,
    // ASSET0_DECINALS: u8 = 1u8,
    // ASSET1_DECINALS: u8 = 1u8,
}

// impl U128 {
//     fn from_uint(value: u64) -> U128 {
//         U128 {upper: value, lower: 0}
//     }
// }

fn main() -> bool {
    if input_count() == 2u8 {
        if input_owner(0).unwrap() == Address::from(MAKER)
            || input_owner(1).unwrap() == Address::from(MAKER)
        {
            return true;
        };
    };

    //FIXME uncomment when this will be released https://github.com/FuelLabs/sway/pull/4526
    let asset0_amount = output_coin_amount(2); // 1000 USDC
    let asset1_amount = output_coin_amount(0); // 200 UNI
    let exp = 1_000_000;
    let price = asset1_amount * exp / asset0_amount;
    assert(price >= PRICE);
    // let price_decimals = 9;
    // let asset0_amount = U128::from_uint(output_coin_amount(2)); // 1000 USDC = 1000 * 1e6 = 1000_000_000
    // let asset1_amount = U128::from_uint(output_coin_amount(0)); // 200 UNI = 200 * 1e9 = 200_000_000_000
    // let exp = U128::from_uint(price_decimals + ASSET0_DECINALS - ASSET1_DECINALS);
    // let price = asset1_amount * U128::from_uint(10).pow(exp) / asset0_amount;
    // assert(price.as_u64().unwrap() >= PRICE);
    assert(PRICE > 0 && ASSET0 != ZERO_B256 && ASSET1 != ZERO_B256 && MAKER != ZERO_B256);
    assert(input_coin_asset_id(0) == ASSET0);
    assert(output_coin_asset_id(0) == ASSET1);
    match output_type(0) {
        Output::Coin => (),
        _ => revert(0),
    };
    assert(verify_output_coin(0));
    assert(output_coin_to(0) == MAKER);
    true
}
