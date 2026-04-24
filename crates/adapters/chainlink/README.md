Chainlink Data Streams adapter for NautilusTrader.

This crate provides a live data-only adapter that maps Chainlink Data Streams
reports into Nautilus custom data events keyed by `feed_id`.

Rust `LiveNode` usage:

```rust
use nautilus_chainlink::{
    config::ChainlinkDataClientConfig,
    factories::ChainlinkDataClientFactory,
    types::ChainlinkData,
};
use nautilus_common::actor::DataActor;
use nautilus_live::node::LiveNode;
use nautilus_model::{
    data::DataType,
    identifiers::{ClientId, TraderId},
};

let client_factory = ChainlinkDataClientFactory::new();
let client_config = ChainlinkDataClientConfig::builder()
    .feed_ids(vec!["0x...feed_id...".to_string()])
    .build();

let mut node = LiveNode::builder(TraderId::from("TESTER-001"), Environment::Live)?
    .with_name("CHAINLINK-NODE")
    .add_data_client(None, Box::new(client_factory), Box::new(client_config))?
    .build()?;

// In a DataActor/Strategy on_start:
let feed_id = "0x...feed_id...";
let data_type = ChainlinkData::data_type(feed_id);
self.subscribe_data(data_type, Some(ClientId::new("CHAINLINK")), None);
```

Python `LiveNode.builder(...)` usage:

```python
from nautilus_trader.adapters.chainlink import (
    ChainlinkData,
    ChainlinkDataClientConfig,
    ChainlinkDataClientFactory,
)
from nautilus_trader.live import LiveNode
from nautilus_trader.model import TraderId
from nautilus_trader.model.data import DataType

builder = LiveNode.builder("CHAINLINK-NODE", TraderId("TESTER-001"), Environment.LIVE)
builder = builder.add_data_client(
    None,
    ChainlinkDataClientFactory(),
    ChainlinkDataClientConfig(feed_ids=["0x...feed_id..."]),
)
node = builder.build()

data_type = DataType(ChainlinkData, {"feed_id": "0x...feed_id..."}, "0x...feed_id...")
```
