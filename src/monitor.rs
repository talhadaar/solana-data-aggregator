use eyre::Result;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_program::slot_history::Slot;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

pub trait Streamer<T> {
    fn next(&mut self) -> impl std::future::Future<Output = Option<T>> + Send;
}

pub struct SlotMonitor {
    client: Arc<PubsubClient>,
    sender: UnboundedSender<Slot>,
    pub receiver: UnboundedReceiver<Slot>,
}

impl SlotMonitor {
    pub async fn new(wss_url: &str) -> Result<Self> {
        let client = Arc::new(PubsubClient::new(wss_url).await?);
        let (sender, receiver) = unbounded_channel();
        Ok(Self {
            client,
            sender,
            receiver,
        })
    }

    pub async fn start_monitoring(&self, token: CancellationToken) -> Result<()> {
        // create subscription
        let (mut sub, unsub) = self.client.slot_subscribe().await?;

        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    // If cancellation occurs, unsubscribe and return
                    unsub().await;
                    break;
                }
                Some(slot_info) = sub.next() => {
                    // If a slot notification is received, queue the slot for processing
                    self.sender.send(slot_info.root)?;
                }
            }
        }

        Ok(())
    }
}

impl Streamer<Slot> for SlotMonitor {
    fn next(&mut self) -> impl std::future::Future<Output = Option<Slot>> + Send {
        self.receiver.recv()
    }
}
