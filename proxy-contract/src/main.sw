contract;

mod math;

use math::*;

use std::logging::log;
use std::call_frames::msg_asset_id;
use std::context::msg_amount;
use std::asset::transfer_to_address;
use std::constants::ZERO_B256;
use std::u128::U128;

configurable {
    BASE_ASSET: AssetId = AssetId::from(ZERO_B256),
    QUOTE_ASSET: AssetId = AssetId::from(ZERO_B256),
    BASE_ASSET_DECIMALS: u32 = 9,
    QUOTE_ASSET_DECIMALS: u32 = 9,
    PRICE_DECIMALS: u32 = 9,
}

abi ProxyContract {
    #[payable]
    fn create_order(
        price: u64,
        predicate_root: Address,
        min_fulfill_base_amount: Option<u64>,
    );
}

enum OrderType {
    SELL: (),
    BUY: (),
}
enum Errors {
    InvalidPayment: (),
    InvalidPredicateRoot: (),
}

struct CreateOrderEvent {
    predicate_root: Address,
    maker: Identity,
    price: u64, //quote_asset_price / base_asset_price * 10.pow(9 + base_asset_decimals - quote_asset_decimals)
    base_asset: AssetId,
    quote_asset: AssetId,
    base_amount: u64,
    order_type: OrderType,
    min_fulfill_base_amount: Option<u64>,
}

impl ProxyContract for Contract {
    #[payable]
    fn create_order(
        price: u64,
        predicate_root: Address,
        min_fulfill_base_amount: Option<u64>,
    ) {
        let payment_amount = msg_amount();
        let payment_asset = msg_asset_id();
        let maker = msg_sender().unwrap();

        require(
            payment_asset == BASE_ASSET || payment_asset == QUOTE_ASSET,
            Errors::InvalidPayment,
        );
        require(
            predicate_root != Address::from(ZERO_B256),
            Errors::InvalidPredicateRoot,
        );

        let (base_amount, order_type) = if payment_asset == BASE_ASSET {
            (payment_amount, OrderType::SELL)
        } else {
            (quote_to_base_amount(payment_amount, price), OrderType::BUY)
        };

        log(CreateOrderEvent {
            predicate_root,
            maker,
            price,
            base_asset: BASE_ASSET,
            quote_asset: QUOTE_ASSET,
            base_amount,
            order_type,
            min_fulfill_base_amount,
        });
        transfer_to_address(predicate_root, payment_asset, payment_amount);
    }
}

// fn base_to_quote_amount(amount: u64, price: u64) -> u64 {
//     amount.mul_div(
//         price,
//         10_u64
//             .pow(BASE_ASSET_DECIMALS + PRICE_DECIMALS - QUOTE_ASSET_DECIMALS),
//     )
// }


fn quote_to_base_amount(amount: u64, price: u64) -> u64 {
    amount.mul_div(
        10_u64
            .pow(BASE_ASSET_DECIMALS + PRICE_DECIMALS - QUOTE_ASSET_DECIMALS),
        price,
    )
}
