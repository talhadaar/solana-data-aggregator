use crate::error::*;
use crate::{traits::BlockStream, types::*};
use solana_client::client_error::ClientErrorKind;
use solana_client::rpc_request::RpcError;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcBlockConfig};
use solana_program::{clock::Slot, system_program::ID};
use solana_transaction_status::{
    EncodedTransaction, EncodedTransactionWithStatusMeta, UiConfirmedBlock, UiInstruction,
    UiMessage, UiParsedInstruction,
};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_util::sync::CancellationToken;

/// [Reference](https://support.quicknode.com/hc/en-us/articles/16459608696721-Solana-RPC-Error-Code-Reference)
const BLOCK_NOT_AVAILABLE: i64 = -32004;
/// [Reference](https://support.quicknode.com/hc/en-us/articles/16459608696721-Solana-RPC-Error-Code-Reference)
const SLOT_SKIPPED: i64 = -32007;

pub fn parse_instruction(instruction: &UiInstruction) -> Option<Transaction> {
    if let UiInstruction::Parsed(UiParsedInstruction::Parsed(parsed_instruction)) = instruction {
        if parsed_instruction.program_id == ID.to_string()
            && parsed_instruction.parsed.get("type")?.as_str()? == "transfer"
        {
            let info = parsed_instruction.parsed.get("info")?.as_object()?;
            return Some(Transaction {
                source: info.get("source")?.as_str()?.to_string(),
                destination: info.get("destination")?.as_str()?.to_string(),
                amount: info.get("lamports")?.as_number()?.as_u64()?,
            });
        }
    }
    None
}

pub fn parse_transaction(
    transaction: EncodedTransactionWithStatusMeta,
) -> Option<Vec<Transaction>> {
    let transaction = match transaction.transaction {
        EncodedTransaction::Json(transaction) => transaction,
        _ => return None,
    };

    let message = match &transaction.message {
        UiMessage::Parsed(message) => message,
        _ => return None,
    };

    let mut transactions = Vec::new();
    for instruction in &message.instructions {
        if let Some(transaction) = parse_instruction(instruction) {
            transactions.push(transaction)
        }
    }
    Some(transactions)
}

impl From<UiConfirmedBlock> for Block {
    fn from(block: UiConfirmedBlock) -> Self {
        let mut transactions: Vec<Transaction> = Vec::new();
        if let Some(block_transactions) = block.transactions {
            for transaction in block_transactions {
                if let Some(mut parsed) = parse_transaction(transaction) {
                    transactions.append(&mut parsed);
                }
            }
        };
        Self {
            height: block.block_height.unwrap(),
            hash: block.blockhash,
            transactions,
            timestamp: block.block_time.unwrap(),
        }
    }
}

/// Fetches block data and parses it into storeable types
pub struct Streamer {
    client: Arc<RpcClient>,
    block_config: Arc<RpcBlockConfig>,
    slot_monitor: UnboundedReceiver<Slot>,
    token: CancellationToken,
}

impl Streamer {
    pub async fn new(
        rpc_url: &str,
        token: CancellationToken,
        slot_monitor: UnboundedReceiver<Slot>,
        block_config: RpcBlockConfig,
    ) -> Result<Self> {
        let client = RpcClient::new(rpc_url.to_string());
        log::debug!("Streamer: RpcClient created");
        Ok(Self {
            client: Arc::new(client),
            block_config: Arc::new(block_config),
            slot_monitor,
            token,
        })
    }

    pub async fn fetch_block(&self, slot: Slot) -> Result<Block> {
        let block = self
            .client
            .get_block_with_config(slot, *self.block_config)
            .await
            .map_err(|error| {
                if let ClientErrorKind::RpcError(rpc_error) = error.kind() {
                    if let RpcError::RpcResponseError { code, .. } = rpc_error {
                        if code == &BLOCK_NOT_AVAILABLE {
                            return Error::SlotMissing(slot);
                        }
                        if code == &SLOT_SKIPPED {
                            return Error::SlotSkipped(slot);
                        }
                    }
                }
                Error::RpcError(error)
            });

        match block {
            Ok(block) => Ok(Block::from(block)),
            Err(e) => Err(e),
        }
    }
}

impl BlockStream for Streamer {
    async fn next(&mut self) -> StreamerResult {
        loop {
            if self.token.is_cancelled() {
                log::info!("TERMINATING");
                return StreamerResult::Error(Error::Termination);
            }

            return match self.slot_monitor.recv().await {
                Some(slot) => match self.fetch_block(slot).await {
                    Ok(block) => StreamerResult::Block(block),
                    Err(e) => StreamerResult::Error(e),
                },
                None => StreamerResult::EOS(),
            };
        }
    }
}
