library;
use std::constants::ZERO_B256;

////////////
// Inuput //
////////////
// const GTF_INPUT_COIN_AMOUNT = 0x105;
// const GTF_INPUT_COIN_ASSET_ID = 0x106;

// pub fn input_coin_asset_id(index: u64) -> b256 {
//     __gtf::<b256>(index, GTF_INPUT_COIN_ASSET_ID)
// }
// pub fn input_coin_amount(index: u64) -> u64 {
//     __gtf::<u64>(index, GTF_INPUT_COIN_AMOUNT)
// }


////////////
// OUTPUT //
////////////

const GTF_OUTPUT_TYPE = 0x201;
const OUTPUT_TYPE_COIN = 0u8; 
const GTF_OUTPUT_COIN_TO: u64 = 0x202;
// const GTF_OUTPUT_COIN_AMOUNT: u64 = 0x203;
// const GTF_OUTPUT_COIN_ASSET_ID: u64 = 0x204;
pub fn verify_output_coin(index: u64) -> bool {
    __gtf::<u64>(index, GTF_OUTPUT_TYPE) == OUTPUT_TYPE_COIN
}

// pub fn output_coin_asset_id(index: u64) -> b256 {
//     __gtf::<b256>(index, GTF_OUTPUT_COIN_ASSET_ID)
// }
// pub fn output_coin_amount(index: u64) -> u64 {
//     __gtf::<u64>(index, GTF_OUTPUT_COIN_AMOUNT)
// }
pub fn output_coin_to(index: u64) -> b256 {
    __gtf::<b256>(index, GTF_OUTPUT_COIN_TO)
}