predicate;

// 🔴 SELL PREDICATE

use std::u128::U128;
use std::outputs::{Output, output_amount, output_count, output_type, output_asset_to};
use std::inputs::{input_amount, input_asset_id, input_count, input_coin_owner};
use std::constants::{ZERO_B256};

// Maker(Alice) wants to exchange BTC for USDC
// Taker(Bob) wants to exchange USDC for BTC 

configurable {
    BASE_ASSET: b256 = ZERO_B256, // Asset that provides maker(Alice)
    QUOTE_ASSET: b256 = ZERO_B256, // Asset that provides taker(Bob)
    MAKER: Address = Address::from(ZERO_B256), // Order owner
    PRICE: u64 = 0, 
    QUOTE_DECIMALS: u8 = 9,
    BASE_DECIMALS: u8 = 9,
    PRICE_DECIMALS: u8 = 9, // optional
    MIN_FULFILL_QUOTE_AMOUNT: u64 = 1, // optional
}


fn main() -> bool {
    true
}
