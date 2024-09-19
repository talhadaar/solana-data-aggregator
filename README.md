## Soalana Data Aggregator

A Rust based application that fetches and aggregates data from solana blockchain.

## Limitations

- For someone having worked withing other rust frameworks only(Solana BPF, CasperLabs and FRAME/Substrate),
  this was an introduction to Solana's SDK and web development. There may be better ways to accomplish this task,
  altough, I have made decisions to the best of my new found knowledge.
- This app only records native Sol transactions for simplicity, data structures and parsers could be extended to decode and record more types of transactions.
- I don't believe testing coverage is adequate enough, and for the sake of time I have decided to skip some tests.

## Design

### Considerations
For design, I took heed of the [5 V's](https://www.techtarget.com/searchdatamanagement/definition/5-Vs-of-big-data) of big data
- **Velocity:** Solana generates a slot, and possibly blocks, every 400ms approx, it's crucial that the
  system is notified of them ASAP, in that regard we chose message passing over chron or scheduled tasks.
- **Volume:** As the incoming data volume from each block could be quite large, we'd prefer to handle fetching, parsing and storage operations in a non blocking thread without having to `.await`.
- **Value:** For this particular solution, we value the native SOL transactions only.
- **Variety:** Application fetches an entire block and it's transactions, but parsing them is a manual and time consuming process, which we have skipped. I could not find types in `solana-sdk` crate to decode transactions/instructions into. 
- **Veracity:** Data is low on veracity, as we do not take historical data into account. Account balances can be negative, if only thing we recorded was outbound SOL transfers.

### Arcitecture
The implementation is split into 5 parts.

#### 1. SlotMonitor

[`SlotMonitor`](https://github.com/talhadaar/solana-data-aggregator/blob/main/src/monitor.rs) make a WS subscription to receive [`slotNotifications`](https://solana.com/docs/rpc/websocket/slotsubscribe) from solana, every time a slot is processed by a validator, and passes this notification as a message into an [`mpsc`](https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.channel.htmlchannel) channel.

#### 2. Streamer
[`Steamer`](https://github.com/talhadaar/solana-data-aggregator/blob/c7c8741e0c2c7bb75225c0c37347d38dd758fdbe/src/streamer.rs#L85) Implementes the [`BlockStream`](https://github.com/talhadaar/solana-data-aggregator/blob/c7c8741e0c2c7bb75225c0c37347d38dd758fdbe/src/streamer.rs#L128) trait, which, checks the mpsc channel for `slotNotification`, and if one is received, fetches, parses and returns related block with it's `async fn next(&mut self)` method

```rust
#[trait_variant::make(Send)]
pub trait BlockStream {
    async fn next(&mut self) -> StreamerResult;
}
```

#### 3. Database
[`Database`](https://github.com/talhadaar/solana-data-aggregator/blob/c7c8741e0c2c7bb75225c0c37347d38dd758fdbe/src/storage.rs#L29) is an abstraction over our storage solution, implementes the [`Storage`](https://github.com/talhadaar/solana-data-aggregator/blob/c7c8741e0c2c7bb75225c0c37347d38dd758fdbe/src/traits.rs#L19) trait.
```rust
#[trait_variant::make(Send)]
pub trait Storage {
    async fn add_block(&mut self, block: &Block) -> Result<()>;
    async fn get_transactions(&self, address: Address) -> Result<Vec<Transaction>>;
    async fn get_account(&self, address: &Address) -> Result<Account>;
}
```

#### 4. Aggregater
[`Aggregator`](https://github.com/talhadaar/solana-data-aggregator/blob/main/src/aggregator.rs) encapsulates types with `Streamer` and `Storage` traits, asks streamer for a new block if there is one and puts it into storage.

### 5. API

A simple warp based REST API serving two endpoints.

#### GET /account?address
Returns with given account's information. For our purpose, SOL balance only.

**Example**
```bash
curl 127.0.0.1:8080/account?address=NvHxHtCXQxsHnUayuKb3yhjRxN9vXChEcDekKNNCE3T
{
    "address":"BhN2e75JhW3mJH4S88kkL4xfjf6j6M2sNhyT6yXBXvr8",
    "balance":93213
}
```

#### GET /transactions?address
Returns with all SOL native transactions made by this address.

**Example**
```bash
curl 127.0.0.1:8080/transactions?address=tKeYE4wtowRb8yRroZShTipE18YVnqwXjsSAoNsFU6g
[
  {
    "source": "tKeYE4wtowRb8yRroZShTipE18YVnqwXjsSAoNsFU6g",
    "destination": "84YKYKo7qN54VHFLn6Eo5uBZMKzUY5Q9qB2t1L3drUeQ",
    "amount": 731
  },
  {
    "source": "tKeYE4wtowRb8yRroZShTipE18YVnqwXjsSAoNsFU6g",
    "destination": "84YKYKo7qN54VHFLn6Eo5uBZMKzUY5Q9qB2t1L3drUeQ",
    "amount": 421
  },
  {
    "source": "tKeYE4wtowRb8yRroZShTipE18YVnqwXjsSAoNsFU6g",
    "destination": "84YKYKo7qN54VHFLn6Eo5uBZMKzUY5Q9qB2t1L3drUeQ",
    "amount": 472
  },
  {
    "source": "tKeYE4wtowRb8yRroZShTipE18YVnqwXjsSAoNsFU6g",
    "destination": "84YKYKo7qN54VHFLn6Eo5uBZMKzUY5Q9qB2t1L3drUeQ",
    "amount": 3
  },
  {
    "source": "tKeYE4wtowRb8yRroZShTipE18YVnqwXjsSAoNsFU6g",
    "destination": "84YKYKo7qN54VHFLn6Eo5uBZMKzUY5Q9qB2t1L3drUeQ",
    "amount": 109
  },
  ...
]
```

## Installation

### Prerequisites

- Need to have Rust and Cargo installed [here](https://www.rust-lang.org/tools/install).
- Ensure you have rust nightly toolchain
```bash
rustup default nightly
```

### Install and run

```bash
# clone the repo
git clone https://github.com/talhadaar/solana-data-aggregator
# move into project repo
cd solana-data-aggregator
# launch project
RUST_LOG=debug cargo run -- -s 127.0.0.1:8080 -r https://damp-few-replica.solana-devnet.quiknode.pro/bb864ce02bee463a190907961fe48e4c7cf5385b -w wss://damp-few-replica.solana-devnet.quiknode.pro/bb864ce02bee463a190907961fe48e4c7cf5385b -d /tmp/solana-data-aggregator.json
# unit testing
cargo test
```

## Command line interface 

```
Solana Data Aggregator

Usage: solana-data-aggregator [OPTIONS]

Options:
  -s, --socket <SOCKET>
          Socket for our REST API

          [default: 127.0.0.1:8080]

  -r, --rpc-provider <RPC_PROVIDER>
          RPC provider URL

  -w, --wss-provider <WSS_PROVIDER>
          WSS Provider URL

  -d, --db-path <DB_PATH>
          Path for our JSON DB file e.g. /tmp/solana_data_aggregator.json

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Deliverables
Some notes on deliverables required.

### 1. Data Aggregator Application
- Application is a CLI implemented with clap, and will run with a single command.
- Its structured into self contained modules and uses trait constraits.
### 2. Documentation 
- A README.md is provided for an overview and usage.
- Inline comments facilitate better understanding of the system and suggest potential improvements.
### 3. Testing
- Unit tests are provided, however the coverage is not production worthy due to a lack of time.
- `request` crate addition creates dependency issues, as a consequence, unit tests on the REST API are commented out.
  
## License
This project is licensed under the MIT License - see the LICENSE file for details.