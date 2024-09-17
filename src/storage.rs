use crate::error::*;
use crate::traits::Storage;
use crate::types::*;
use nanodb::nanodb::NanoDB;
use serde::Deserialize;
use serde::Serialize;
use solana_program::clock::Slot;
use std::fmt::Display;

pub const LATEST_BLOCKHEIGHT_KEY: &str = "latest_bh";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DbKey {
    TransactionIndex = 1,
    AccountBalance = 2,
    Block = 3,
}

pub fn db_key<T: Display>(key_type: DbKey, key: &T) -> String {
    format!("{:?}-{}", key_type, key)
}

pub struct ChainMedadata {
    pub last_slot: Slot,
    pub last_block_height: u64,
}

pub struct Database(NanoDB);
impl Database {
    pub fn new(path: &str) -> Self {
        let db = NanoDB::open(path).unwrap();
        Self(db)
    }
}

impl Storage for Database {
    async fn add_block(&mut self, block: &Block) -> Result<()> {
        let block_key = db_key(DbKey::Block, &block.height);
        if self.0.data().await.get(&block_key).is_ok() {
            return Err(Error::StorageError(format!(
                "Block {:?} exists",
                block.height
            )));
        }

        for (index, transaction) in block.transactions.iter().enumerate() {
            // Record transactions done by sender
            let tx_index = TransactionIndex {
                block_height: block.height,
                index,
            };

            let mut sender_index = match self
                .0
                .data()
                .await
                .get(db_key(DbKey::TransactionIndex, &transaction.source).as_ref())
            {
                Ok(sender_index) => sender_index
                    .into::<Vec<TransactionIndex>>()
                    .unwrap_or_default(),
                Err(_) => Vec::new(),
            };

            if !sender_index.contains(&tx_index) {
                sender_index.push(tx_index);
                self.0
                    .insert(
                        db_key(DbKey::TransactionIndex, &transaction.source).as_str(),
                        &sender_index,
                    )
                    .await?;
            }

            let sender_balance_key = db_key(DbKey::AccountBalance, &transaction.source);
            let sender_balance = match self.0.data().await.get(&sender_balance_key) {
                Ok(sender_balance) => sender_balance.into::<i64>().unwrap_or_default(),
                Err(_) => 0,
            };

            let receiver_balance_key = db_key(DbKey::AccountBalance, &transaction.destination);
            let receiver_balance = match self.0.data().await.get(&receiver_balance_key) {
                Ok(receiver_balance) => receiver_balance.into::<i64>().unwrap_or_default(),
                Err(_) => 0,
            };

            self.0
                .insert(
                    sender_balance_key.as_ref(),
                    sender_balance - transaction.amount as i64,
                )
                .await?;
            self.0
                .insert(
                    receiver_balance_key.as_ref(),
                    receiver_balance + transaction.amount as i64,
                )
                .await?;
        }
        self.0.insert(&block_key, block).await?;
        Ok(())
    }

    async fn get_transactions(&self, address: Address) -> Result<Vec<Transaction>> {
        let tx_index = match self
            .0
            .data()
            .await
            .get(db_key(DbKey::TransactionIndex, &address).as_ref())
        {
            Ok(tx_index) => tx_index.into::<Vec<TransactionIndex>>().unwrap_or_default(),
            Err(_) => {
                return Err(Error::StorageError(format!(
                    "No transactions for {}",
                    address
                )))
            }
        };

        let mut transactions = Vec::new();
        for index in tx_index {
            let block_key = db_key(DbKey::Block, &index.block_height);
            let block: Block = self
                .0
                .data()
                .await
                .get(&block_key)
                .map_err(|_| {
                    Error::StorageError(format!("Block {:?} not found", index.block_height))
                })?
                .into::<Block>()
                .unwrap();

            transactions.push(block.transactions[index.index].clone());
        }

        Ok(transactions)
    }

    async fn get_account(&self, address: &Address) -> Result<Account> {
        let balance = match self
            .0
            .data()
            .await
            .get(db_key(DbKey::AccountBalance, address).as_ref())
        {
            Ok(balance) => balance.into::<i64>().unwrap_or_default(),
            Err(_) => {
                return Err(Error::StorageError(format!(
                    "Account {} not found",
                    address
                )))
            }
        };

        Ok(Account {
            address: address.clone(),
            balance,
        })
    }
}
