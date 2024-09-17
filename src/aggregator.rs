use crate::error::*;
use crate::traits::*;
use crate::types::StreamerResult;
use tokio_util::sync::CancellationToken;

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
                    // TODO should be possible to move this out to a separate thread and monitor resolution of future,
                    // rather than blocking the streamer thread
                    // will improve data velocity
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
                StreamerResult::EOS() => {
                    log::debug!("EOS");
                }
            }
        }
    }
}
