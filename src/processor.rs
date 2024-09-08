use crate::{
    monitor::{SlotMonitor, Streamer},
    storage::StorageInterface,
};
use eyre::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_transaction_status::EncodedConfirmedBlock;
use std::sync::Arc;

pub struct Processor {
    monitor: SlotMonitor,
    client: Arc<RpcClient>,
    db: StorageInterface,
}

impl Processor {
    pub async fn new(rpc_url: String, monitor: SlotMonitor, db: StorageInterface) -> Self {
        let client = Arc::new(RpcClient::new(rpc_url));
        Self {
            monitor,
            client,
            db,
        }
    }

    pub async fn process_slot_notifications(&mut self) -> Result<()> {
        if let Some(slot) = self.monitor.next().await {
            // fetch block of this slot
            let block = self.client.get_block(slot).await?;

            // spawn a task to process and store
        }
        Ok(())
    }

    pub async fn process_block(block: &EncodedConfirmedBlock) -> Result<()> {
        // store block in db if it has transactions
        Ok(())
    }
}
