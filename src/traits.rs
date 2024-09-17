use crate::error::*;
use crate::types::*;

pub trait Stream<T> {
    fn next(&mut self) -> impl std::future::Future<Output = Option<T>> + Send;
}

/// Returns latest produced block on Solana
/// async because needs might need to do some async operations within it's scope
/// Not expected to return a future
#[trait_variant::make(Send)]
pub trait BlockStream {
    async fn next(&mut self) -> StreamerResult;
}

/// Abstraction over database storage
/// Monitors an [ActionsQueueRx] for new DB operations and executes them on the DB
#[trait_variant::make(Send)]
pub trait Storage {
    /// Monitors an [ActionsQueueRx] for new DB operations
    // async fn serve_queue(&self, actions_queue: ActionsQueueRx) -> Result<()>;
    /// Processes an action received from the [ActionsQueueRx]
    // async fn process_action(&self, action: Action) -> Result<()>;
    async fn add_block(&mut self, block: &Block) -> Result<()>;
    async fn get_transactions(&self, address: Address) -> Result<Vec<Transaction>>;
    async fn get_account(&self, address: &Address) -> Result<Account>;
}

// /// Abstraction over the [Storage] trait for the [Aggregator]
// /// Adds actions to the [ActionsQueueTx] for the [Storage] to process
// /// Awaits for [Storage] to process the action and returns the result
// pub trait ActionsQueue {
//     fn new(actions_queue: ActionsQueueTx) -> Self;
//     async fn add_block(&self, block: Block) -> ActionResult;
//     async fn get_account(&self, address: Address) -> ActionResult;
//     async fn get_transactions(&self, address: Address) -> ActionResult;
// }
