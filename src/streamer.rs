use crate::error::*;
use crate::{traits::BlockStream, types::*};
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcBlockConfig};
use solana_program::{clock::Slot, system_program::ID};
use solana_transaction_status::{
    EncodedTransaction, EncodedTransactionWithStatusMeta, UiConfirmedBlock, UiInstruction,
    UiMessage, UiParsedInstruction,
};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_util::sync::CancellationToken;

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
        rpc_url: String,
        token: CancellationToken,
        slot_monitor: UnboundedReceiver<Slot>,
        block_config: RpcBlockConfig,
    ) -> Result<Self> {
        let client = RpcClient::new(rpc_url);
        Ok(Self {
            client: Arc::new(client),
            block_config: Arc::new(block_config),
            slot_monitor,
            token,
        })
    }

    pub async fn fetch_block(&self, slot: Slot) -> Result<Block> {
        let block = match self
            .client
            .get_block_with_config(slot, *self.block_config)
            .await
        {
            Ok(block) => block,
            Err(e) => return Err(Error::RpcError(e)),
        };
        Ok(Block::from(block))
    }
}

impl BlockStream for Streamer {
    async fn next(&mut self) -> StreamerResult {
        loop {
            if self.token.is_cancelled() {
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
