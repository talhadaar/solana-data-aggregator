use crate::error::*;
use crate::types::*;
use solana_program::clock::Slot;

pub trait Stream<T> {
    fn next(&mut self) -> impl std::future::Future<Output = Option<T>> + Send;
}

/// Returns an slot notification from monitor's queue of type [UnboundedReceiver<T>]
/// Returns None if monitor is terminated or there was no slot notification
pub trait Monitor<T> {
    fn next(&mut self) -> impl std::future::Future<Output = Option<T>> + Send;
}

#[allow(async_fn_in_trait)]
/// Returns latest produced block on Solana
/// async because needs might need to do some async operations within it's scope
/// Not expected to return a future
pub trait BlockStream {
    async fn next(&mut self) -> StreamerResult;
}

#[allow(async_fn_in_trait)]
pub trait Storage {
    async fn add_block(&self, block: &Block) -> Result<()>;
    async fn get_block(&self, height: Slot) -> Result<Block>;
    async fn get_transactions(&self, height: Slot) -> Result<Transaction>;
    async fn get_accounts(&self, address: &Address) -> Result<Account>;
}
