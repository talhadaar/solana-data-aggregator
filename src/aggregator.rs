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
        Self {
            streamer,
            token,
            storage,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            if self.token.is_cancelled() {
                return Err(Error::Termination);
            }

            match self.streamer.next().await {
                StreamerResult::Block(block) => {
                    self.storage.add_block(&block).await?;
                }
                StreamerResult::Error(error) => {
                    return Err(error);
                }
                StreamerResult::EOS() => {
                    break;
                }
            }
        }
        Ok(())
    }
}
