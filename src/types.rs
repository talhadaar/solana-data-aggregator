/// TODO could create interface traits for Transaction, Block and Account types
/// So that we could enforce what information is required for each type to contain
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

pub type Hash = String;
pub type Address = String;

pub type ActionsQueueRx = mpsc::Receiver<Action>;
pub type ActionsQueueTx = mpsc::Sender<Action>;

pub enum StreamerResult {
    Block(Block),
    EOS(),
    Error(Error),
}

pub type AddBlockResult = Result<()>;
pub type GetTransactionsResult = Result<Vec<Transaction>>;
pub type GetAccountsResult = Result<Account>;

pub enum Action {
    AddBlock(Block, oneshot::Sender<AddBlockResult>),
    GetTransactions(Address, oneshot::Sender<GetTransactionsResult>),
    GetAccounts(Address, oneshot::Sender<GetAccountsResult>),
}

impl Action {
    pub async fn send(self, sender: mpsc::Sender<Action>, source: &str) -> Result<()> {
        sender
            .send(self)
            .await
            .map_err(|error| Error::ChannelFailed(source.to_string(), error.to_string()))?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
pub struct Transaction {
    pub source: Address,
    pub destination: Address,
    pub amount: u64,
}

/// TODO make Block type generic over the type of transactions it contains by trait constraints
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub height: u64,
    pub hash: Hash,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
}

/// TODO make Account type generic over the type of Address it contains
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
pub struct Account {
    pub address: Address,
    pub balance: i64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
pub struct TransactionIndex {
    pub block_height: u64,
    pub index: usize,
}
