use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

pub type Hash = String;
pub type Address = String;

pub enum StreamerResult {
    Block(Block),
    EOS(),
    Error(Error),
}

pub enum ActionsResult {
    BlockAdded(Result<()>),
    GetAccounts(Result<Vec<Account>>),
    GetTransactions(Result<Vec<Transaction>>),
}

pub enum Actions {
    AddBlock(Block, oneshot::Sender<ActionsResult>),
    GetTransactions(ActionsResult),
    GetAccounts(ActionsResult),
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Account {
    pub address: Address,
    pub balance: i64,
}
