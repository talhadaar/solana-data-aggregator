/// TODO could create interface traits for Transaction, Block and Account types
/// So that we could enforce what information is required for each type to contain


use crate::error::Error;
use serde::{Deserialize, Serialize};

pub type Hash = String;
pub type Address = String;

// pub type ActionsQueueRx = mpsc::UnboundedReceiver<Action>;
// pub type ActionsQueueTx = mpsc::UnboundedSender<Action>;
// pub type SlotMonitorRx = mpsc::UnboundedReceiver<Slot>;
// pub type SlotMonitorTx = mpsc::UnboundedSender<Slot>;

pub enum StreamerResult {
    Block(Block),
    EOS(),
    Error(Error),
}

// pub enum ActionResult {
//     BlockAdded(Result<()>),
//     GetAccounts(Result<Vec<Account>>),
//     GetTransactions(Result<Vec<Transaction>>),
// }

// pub enum Action {
//     AddBlock(Block, oneshot::Sender<ActionResult>),
//     GetTransactions(ActionResult),
//     GetAccounts(ActionResult),
// }

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
