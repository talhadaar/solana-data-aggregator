use crate::types::*;
use rocksdb::{Options, DB};
use solana_program::clock::Slot;
use std::sync::{Arc, Mutex};
pub struct StorageInterface(Arc<Mutex<DB>>);

impl StorageInterface {
    pub fn new(db_path: &String) -> Self {
        let mut options = Options::default();
        options.set_error_if_exists(false);
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        // list existing ColumnFamilies in the given path. returns Err when no DB exists

        // open a DB with specifying ColumnFamilies
        let cfs = vec!["blocks", "accounts", "transactions"];
        let db = rocksdb::DB::open_cf(&options, db_path, cfs).unwrap();

        Self(Arc::new(Mutex::new(db)))
    }

    pub fn add_block(&self, slot: Slot, block: &Block) -> Result<(), rocksdb::Error> {
        let db = self.0.lock().unwrap();
        let cf = db.cf_handle("blocks").unwrap();
        db.put_cf(cf, slot.to_le_bytes(), serde_json::to_vec(block).unwrap())
    }

    pub fn add_transaction(&self, slot: Slot, tx: &Transaction) -> Result<(), rocksdb::Error> {
        let db = self.0.lock().unwrap();
        let cf = db.cf_handle("transactions").unwrap();
        db.put_cf(cf, slot.to_le_bytes(), serde_json::to_vec(tx).unwrap())
    }

    pub fn add_account(&self, account: &Account) -> Result<(), rocksdb::Error> {
        let db = self.0.lock().unwrap();
        let cf = db.cf_handle("accounts").unwrap();
        db.put_cf(cf, &account.address, serde_json::to_vec(account).unwrap())
    }

    pub fn get_block(&self, slot: Slot) -> Result<Option<Block>, rocksdb::Error> {
        let db = self.0.lock().unwrap();
        let cf = db.cf_handle("blocks").unwrap();
        match db.get_cf(cf, slot.to_le_bytes()) {
            Ok(Some(data)) => Ok(Some(serde_json::from_slice(&data).unwrap())),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_transaction(&self, slot: Slot) -> Result<Option<Transaction>, rocksdb::Error> {
        let db = self.0.lock().unwrap();
        let cf = db.cf_handle("transactions").unwrap();
        match db.get_cf(cf, slot.to_le_bytes()) {
            Ok(Some(data)) => Ok(Some(serde_json::from_slice(&data).unwrap())),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_account(&self, address: &String) -> Result<Option<Account>, rocksdb::Error> {
        let db = self.0.lock().unwrap();
        let cf = db.cf_handle("accounts").unwrap();
        match db.get_cf(cf, address) {
            Ok(Some(data)) => Ok(Some(serde_json::from_slice(&data).unwrap())),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
