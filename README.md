# Spark Rust SDK

This repository contains the implementation of a predicate-based order book on the Fuel network, along with tests and Spark Rust SDK code.

## Components
### Predicate + Proxy
This system utilizes a predicate to represent an order that can be filled by anyone. To create an order, the maker sends a coin to the predicate root, which can be unlocked by any transaction that fulfills the order's conditions. These conditions, such as transferring a specific amount of an asset to a receiver, are encoded into the predicate bytecode, making them specific to that particular order.

The order owner can execute the order by spending the predicate coin in a transaction that satisfies the order's conditions. The owner has flexibility in how they use the predicate coin, as long as the transaction output meets the order's conditions and is the first output in the set.

To cancel an order, the maker can spend the predicate coin in a transaction that includes a single coin input signed by the receiver. The transaction consists of two inputs: the signed coin and the predicate coin.

Alice provides information about the price and tokens in this predicate. Additionally, Alice can send additional money to the same predicate root to increase the change amount.

### Spark Rust SDK
Designed for seamless integration with CLOB Spark using the Rust programming language, the Spark Rust SDK offers the following functionality:
----------
#### Getting Predicate Instance
To create a predicate, you need to provide the following configurables:
```rust
configurable {
    ASSET0: b256 = ZERO_B256, // Asset that provides the maker (Alice)
    ASSET1: b256 = ZERO_B256, // Asset that provides the taker (Bob)
    MAKER: b256 = ZERO_B256, // Order owner
    PRICE: u64 = 0, // asset1_amount / asset0_amount
    ASSET0_DECIMALS: u8 = 1,
    ASSET1_DECIMALS: u8 = 1,
    PRICE_DECIMALS: u8 = 9, // optional
    MIN_FULFILL_AMOUNT0: u64 = 1, // optional
}
```
Example:
```rust
let usdc_decimals = 6;
let uni_decimals = 9;
let amount0 = 1000_000_000_u64; // 1000 USDC
let amount1 = 300_000_000_000_u64; // 200 UNI
let price_decimals = 9;

let exp = (price_decimals + usdc_decimals - uni_decimals).into();
let price = amount1 * 10u64.pow(exp) / amount0;
println!("Order price: {:?} UNI/USDC", price);

let configurables = LimitOrderPredicateConfigurables::new()
    .set_ASSET0(Bits256::from_hex_str(&usdc_asset_id.to_string()).unwrap())
    .set_ASSET1(Bits256::from_hex_str(&uni_asset_id.to_string()).unwrap())
    .set_ASSET0_DECIMALS(usdc_decimals)
    .set_ASSET1_DECIMALS(uni_decimals)
    .set_MAKER(Bits256::from_hex_str(&alice.address().hash().to_string()).unwrap())
    .set_PRICE(price)
    .set_PRICE_DECIMALS(price_decimals)
    .set_MIN_FULFILL_AMOUNT0(0);

let predicate: Predicate = Predicate::load_from(PREDICATE_BIN_PATH)
    .unwrap()
    .with_configurables(configurables);
```
----------
#### Order Creation
```rust
async fn create_order(
    wallet: &WalletUnlocked, // Order owner
    proxy_address: &str, // Proxy contract address as a string


    params: ProxySendFundsToPredicateParams, 
    amount: u64, // amount0
) -> Result<FuelCallResponse<()>, fuels::prelude::Error>;
```
Example:
```rust
let params = ProxySendFundsToPredicateParams {
    predicate_root: predicate.address().into(),
    asset_0: usdc.contract_id().into(),
    asset_1: uni.contract_id().into(),
    maker: alice.address().into(),
    min_fulfill_amount_0: 1,
    price,
    asset_0_decimals: 6,
    asset_1_decimals: 9,
    price_decimals: 9,
};

create_order(&alice, PROXY_ADDRESS, params, amount0)
    .await
    .unwrap();
```
----------
#### Order Cancellation
```rust
pub async fn cancel_order(
    wallet: &WalletUnlocked, // Order owner
    predicate: &Predicate, // Predicate instance
    asset0: AssetId,
    amount0: u64,
) -> Result<FuelCallResponse<()>, fuels::prelude::Error>;
```
Example:
```rust
cancel_order(&alice, &predicate, usdc_asset_id, usdc_mint_amount)
    .await
    .unwrap();
```
----------
#### Order Fulfillment
```rust
async fn fulfill_order(
    wallet: &WalletUnlocked, // Taker
    predicate: &Predicate, // Predicate instance
    owner_address: &Bech32Address, 
    asset0: AssetId,
    amount0: u64,
    asset1: AssetId,
    amount1: u64,
) -> Result<FuelCallResponse<()>, fuels::prelude::Error>;
```
Example:
```rust
fulfill_order(
    &bob,
    &predicate,
    alice.address(),
    usdc_asset_id,
    usdc_mint_amount, // amount0
    uni_asset_id,
    uni_mint_amount, // amount0
)
.await
.unwrap();
```
