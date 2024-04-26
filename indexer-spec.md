Here's the translation of your text from Russian to English:

# Specification for the Predicate Orderbook Indexer

Before reading this document, you must read the README file to understand how the predicate orderbook is structured.

### Data Retrieval
Orders will be created using a proxy that will emit an event, which will store information about the order and the predicate root.

```rust
contract;
use std::logging::log;
use std::call_frames::msg_asset_id;
use std::context::msg_amount;
use std::token::transfer_to_address;
use std::constants::ZERO_B256;

abi ProxyContract {
    #[payable]
    fn create_order(params: CreateOrderEvent);
}

struct CreateOrderEvent {
    predicate_root: Address,
    base_asset: ContractId,
    quote_asset: ContractId,
    maker: Address,
    min_fulfill_base_amount: u64,
    price: u64, //quote_asset_price / base_asset_price * 10.pow(9 + base_asset_decimals - quote_asset_decimals)
    base_asset_decimals: u8,
    quote_asset_decimals: u8,
    price_decimals: u8,
}

impl ProxyContract for Contract {
    #[payable]
    fn create_order(params: CreateOrderEvent){
        let amount = msg_amount();
        assert(params.predicate_root != Address::from(ZERO_B256) && params.maker != Address::from(ZERO_B256));
        assert(params.base_asset != ContractId::from(ZERO_B256) && params.quote_asset != ContractId::from(ZERO_B256));
        assert(amount > 0 && msg_asset_id() == params.base_asset);
        assert(params.min_fulfill_base_amount > 0 && params.price > 0);
        assert(params.base_asset_decimals >= 0u8 && params.quote_asset_decimals >= 0u8 && params.price_decimals >= 0u8); //TODO add <= 9 check
        log(params);
        transfer_to_address(amount, params.base_asset, params.predicate_root);
    }
}
```

We will also have an indexer in operation that will monitor events from this contract and log information about orders in a database. This way, we will process the creation of orders.

```ts
// 1. Get receipt data by contract ID
const currentBlock = await this.getSettings();
const fromBlock = currentBlock === 0 ? +START_BLOCK : currentBlock;
const toBlock = fromBlock + STEP;
const receiptsResult = await fetchReceiptsFromEnvio(fromBlock, toBlock, thicontracts);
// 2. Decode receipts and extract events
const receipts = receiptsResult.receipts.filter(({contract_id}) => contract_id == PROXY_ID),
const events = getDecodedLogs(receipts, abi.interface)
// 3. Record events into the CreateOrderEvents table in the database
const decodedEvents = decodeOrderbookReceipts(receipts, abi).sort((a, b) => {
    if (+a.timestamp < +b.timestamp) return -1;
    if (+a.timestamp > +b.timestamp) return 1;
    return 0;
});
for (let eventIndex = 0; eventIndex < decodedEvents.length; eventIndex++) {
    const event: any = decodedEvents[eventIndex];
    if (isEvent("CreateOrderEvent", event, abi)) {
        await CreateOrderEvent.create({...event});
    }
// 4. Log order details into the Order table in the database
    const defaultOrder = getOrderFromCreateEvent(event)
    const [order, created] = await Order.findOrCreate({
        where: {order_id: (event as any).order_id},
        defaults: defaultOrder,
    });
    if (!created) {
        const base_balance = getBalance(defaultOrder.predicate_root, defaultOrder.base_asset);
        await order.set("base_size", defaultOrder == null ? "0" : base_balance).save();

        const quote_balance = getBalance(defaultOrder.predicate_root, defaultOrder.quote_asset);
        await order.set("quote_size", defaultOrder == null ? "0" : quote_balance).save();
    }
}
```

To track the status of orders after they are fulfilled, matched, or canceled, we will employ another indexer that examines all interactions with predicates on the blockchain. This indexer will compare the specific predicate root with entries in our database.

```ts
// 1. Retrieve the latest transactions from the blockchain and filter for predicate transactions
const currentBlock = await this.getSettings();
const fromBlock = currentBlock === 0 ? +START_BLOCK : currentBlock;
const toBlock = fromBlock + STEP;
const txsResult = await fetchTxsFromFuelNode(fromBlock, toBlock, thicontracts);
const predicateTxs = filterPredicateTxs(txsResult)

for(const tx in predicateTxs){
// 2. Search for a predicate root in our database
    if (await checkIfPredicateRootExistsInOurDatabase(tx.predicate_root)){
// 3. If the predicate root exists, update order details (like balance) in the Order table
        const order = await Order.findOrCreate({where: {predicate_root: tx.predicate_root}});
        
        const base_balance = getBalance(defaultOrder.predicate_root, defaultOrder.base_asset);
        await order.set("base_size", defaultOrder == null ? "0" : base_balance).save();

        const quote_balance = getBalance(defaultOrder.predicate_root, defaultOrder.quote_asset);
        await order.set("quote_size", defaultOrder == null ? "0" : quote_balance).save();
    }
}
```
