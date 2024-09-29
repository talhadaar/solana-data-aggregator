use crate::error::*;
use crate::traits::ActionsQueue;
use crate::traits::Storage;
use crate::types::*;
use nanodb::nanodb::NanoDB;
use serde::Deserialize;
use serde::Serialize;
use solana_program::clock::Slot;
use std::fmt::Display;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

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

async fn receive<T>(receiver: oneshot::Receiver<T>, source: &str) -> Result<T> {
    receiver
        .await
        .map_err(|error| Error::ChannelFailed(source.to_string(), error.to_string()))
}

async fn send(sender: ActionsQueueTx, action: Action, source: &str) -> Result<()> {
    sender
        .send(action)
        .await
        .map_err(|err| Error::ChannelFailed(source.to_string(), err.to_string()))
}

const ADD_BLOCK_SOURCE: &str = "add_block";
const GET_ACCOUNTS_SOURCE: &str = "get_accounts";
const GET_TRANSACTIONS_SOURCE: &str = "get_transactions";

#[derive(Clone)]
pub struct StorageInterface(ActionsQueueTx);
impl ActionsQueue for StorageInterface {
    async fn add_block(&mut self, block: Block) -> AddBlockResult {
        let (tx, rx) = oneshot::channel();
        let action = Action::AddBlock(block, tx);
        // send action to actions queue
        send(self.0.clone(), action, ADD_BLOCK_SOURCE).await?;
        receive(rx, ADD_BLOCK_SOURCE).await?
    }

    async fn get_account(&mut self, address: Address) -> GetAccountsResult {
        let (tx, rx) = oneshot::channel();
        let action = Action::GetAccounts(address, tx);
        send(self.0.clone(), action, GET_ACCOUNTS_SOURCE).await?;
        receive(rx, GET_ACCOUNTS_SOURCE).await?
    }
    async fn get_transactions(&mut self, address: Address) -> GetTransactionsResult {
        let (tx, rx) = oneshot::channel();
        let action = Action::GetTransactions(address, tx);
        send(self.0.clone(), action, ADD_BLOCK_SOURCE).await?;
        receive(rx, GET_TRANSACTIONS_SOURCE).await?
    }
}

#[derive(Debug, Clone)]
pub struct Database(NanoDB);
impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let db = NanoDB::open(path)?;
        Ok(Self(db))
    }
}

impl Storage for Database {
    async fn serve_queue(
        &mut self,
        mut actions_queue: ActionsQueueRx,
        token: CancellationToken,
    ) -> Result<()> {
        loop {
            if token.is_cancelled() {
                log::info!("TERMINATION");
                return Err(Error::Termination);
            }

            if let Some(action) = actions_queue.recv().await {
                if let Err(Error::ChannelFailed(a, b)) = self.process_action(action).await {
                    log::error!("Channel failure {:?} {:?}", a, b);
                }
            }
        }
    }

    async fn process_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::AddBlock(block, sender) => {
                if let Err(_) = sender.send(self.add_block(&block).await) {
                    return Err(Error::ChannelFailed(
                        "add_block".to_string(),
                        "send failed".to_string(),
                    ));
                }
            }
            Action::GetAccounts(account, sender) => {
                if let Err(_) = sender.send(self.get_account(&account).await) {
                    return Err(Error::ChannelFailed(
                        "get_account".to_string(),
                        "send failed".to_string(),
                    ));
                }
            }
            Action::GetTransactions(address, sender) => {
                if let Err(_) = sender.send(self.get_transactions(&address).await) {
                    return Err(Error::ChannelFailed(
                        "get_transactions".to_string(),
                        "send failed".to_string(),
                    ));
                }
            }
        };
        Ok(())
    }

    async fn add_block(&mut self, block: &Block) -> AddBlockResult {
        let block_key = db_key(DbKey::Block, &block.height);
        if self.0.data().await.get(&block_key).is_ok() {
            return Err(Error::StorageError(format!(
                "Block {:?} exists",
                block.height
            )));
        }

        for (index, transaction) in block.transactions.iter().enumerate() {
            // Record transactions done by sender in their transaction index
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
        self.0.write().await?;
        Ok(())
    }

    async fn get_transactions(&self, address: &Address) -> GetTransactionsResult {
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

    async fn get_account(&self, address: &Address) -> GetAccountsResult {
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

#[cfg(test)]
mod storage_tests {
    use crate::storage::Database;
    use crate::traits::Storage;
    use crate::types::*;
    use rand::Rng;

    #[tokio::test]
    async fn sanity_check() {
        let path = "/tmp/storage.json";
        let mut db = Database::new(path).unwrap();

        let mut rng = rand::thread_rng();
        let block_height: u64 = rng.gen_range(0..10000000);
        let source = String::from(format!("source{}", block_height));
        let destination = String::from(format!("destination{}", block_height));
        let block_hash = String::from(format!("block_hash{}", block_height));

        let amount = 100;
        let transaction = Transaction {
            source: source.to_string(),
            destination: destination.to_string(),
            amount,
        };

        let block = Block {
            height: block_height,
            transactions: vec![transaction],
            hash: block_hash.to_string(),
            timestamp: 100100,
        };
        db.add_block(&block).await.unwrap();

        let transactions = db.get_transactions(&source).await.unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].amount, amount);
        assert_eq!(transactions[0].source, source);
        assert_eq!(transactions[0].destination, destination);

        let account = db.get_account(&destination).await.unwrap();
        assert_eq!(account.balance, amount as i64);
    }
}
