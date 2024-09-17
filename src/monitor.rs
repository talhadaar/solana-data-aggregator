use crate::error::*;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_program::slot_history::Slot;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct SlotMonitor {
    client: Arc<PubsubClient>,
    sender: UnboundedSender<Slot>,
    token: CancellationToken,
}

impl SlotMonitor {
    pub async fn new(
        wss_url: &str,
        token: CancellationToken,
        monitor_tx: UnboundedSender<Slot>,
    ) -> Result<Self> {
        let client = Arc::new(match PubsubClient::new(wss_url).await {
            Ok(client) => client,
            Err(e) => return Err(Error::PubSubError(e)),
        });
        Ok(Self {
            client,
            sender: monitor_tx,
            token,
        })
    }

    pub async fn start_monitoring(&self) -> Result<()> {
        // create subscription
        let (mut sub, unsub) = match self.client.slot_subscribe().await {
            Ok(sub) => sub,
            Err(e) => return Err(Error::PubSubError(e)),
        };

        loop {
            tokio::select! {
                _ = self.token.cancelled() => {
                    // If cancellation occurs, unsubscribe and return
                    unsub().await;
                    return Err(Error::Termination);
                }
                Some(slot_info) = sub.next() => {
                    // If a slot notification is received, queue the slot for processing
                    match self.sender.send(slot_info.root) {
                        Ok(_) => (),
                        Err(e) => return Err(Error::ChannelFailed("SlotMonitor".to_string(), e.to_string())),
                    }
                }
            }
        }
    }
}

// impl Monitor<Slot> for SlotMonitor {
//     fn next(&mut self) -> impl std::future::Future<Output = Option<Slot>> + Send {
//         self.receiver.recv()
//     }
// }
