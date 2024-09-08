// pub type Result<T> = std::result::Result<T, Error>;
use serde::{Deserialize, Serialize};

pub type Hash = String;
pub type Address = String;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct NativeTransaction {
    pub sender: Address,
    pub receiver: Address,
    pub amount: u64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TransactionWithMeta {
    pub data: NativeTransaction,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub height: u64,
    pub hash: Hash,
    pub timestamp: i64,
    pub transactions: Vec<NativeTransaction>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Account {
    pub address: Address,
    pub balance: i64,
}
