use crate::error::*;
use crate::traits::*;
use crate::types::StreamerResult;
use tokio_util::sync::CancellationToken;

/// TODO should use a multi-producer-multi-consumer channel
/// Each block is fetched and parsed by a Consumer task and sent to another multi-producer-multi-consumer channel
/// Each Aggregator task will take the block from the channel and store
/// This will relieve the need for `Aggregator::run` task to block on self.storage.add_block() operation
pub struct Aggregator<T: BlockStream, S: Storage> {
    pub streamer: T,
    pub token: CancellationToken,
    pub storage: S,
}

impl<T: BlockStream, S: Storage> Aggregator<T, S> {
    pub fn new(streamer: T, token: CancellationToken, storage: S) -> Self {
        log::debug!("Created successfully");
        Self {
            streamer,
            token,
            storage,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            if self.token.is_cancelled() {
                log::info!("TERMINATING");
                return Err(Error::Termination);
            }

            match self.streamer.next().await {
                StreamerResult::Block(block) => {
                    log::info!("Recording block: {:?}", block.height);
                    self.storage.add_block(&block).await?;
                }
                StreamerResult::Error(error) => {
                    // check if a slot was missing or skipped
                    log::warn!("{}", error);
                    if let Error::SlotSkipped(_) | Error::SlotMissing(_) = error {
                        continue;
                    }
                    return Err(error);
                }
                // EOS is not an error, just log and continue
                // Alternatively, we could `sleep` for a while before continuing
                // Not sure how to decide on sleep duration
                StreamerResult::EOS() => {
                    log::debug!("EOS");
                }
            }
        }
    }
}
