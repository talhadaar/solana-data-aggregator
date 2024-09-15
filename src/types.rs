use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use solana_program::clock::Slot;
use tokio::sync::{mpsc, oneshot};

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub height: u64,
    pub hash: Hash,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
}

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
