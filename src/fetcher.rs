use crate::types::*;
use eyre::Result;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcBlockConfig};
use solana_program::{clock::Slot, system_program::ID};
use solana_transaction_status::{
    EncodedTransaction, EncodedTransactionWithStatusMeta, UiConfirmedBlock, UiInstruction,
    UiMessage, UiParsedInstruction,
};
use std::sync::Arc;

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
pub struct Fetcher {
    client: Arc<RpcClient>,
    block_config: Arc<RpcBlockConfig>,
}

impl Fetcher {
    pub fn new(rpc_url: String, block_config: RpcBlockConfig) -> Self {
        let client = RpcClient::new(rpc_url);
        Self {
            client: Arc::new(client),
            block_config: Arc::new(block_config),
        }
    }

    pub async fn fetch_block(&self, slot: Slot) -> Result<Block> {
        let block = self
            .client
            .get_block_with_config(slot, *self.block_config)
            .await?;
        Ok(Block::from(block))
    }
}
