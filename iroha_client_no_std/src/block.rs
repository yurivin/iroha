//! This module contains `Block` structures for each state, it's transitions, implementations and related traits
//! implementations.

use crate::tx::ValidTransaction;
use crate::{
    crypto::{self, Hash, KeyPair, Signatures},
    prelude::*,
};
use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};

/// Message's variants that are used by peers to communicate in the process of consensus.
#[derive(Decode, Encode, Debug, Clone)]
pub enum Message {
    /// Gossip about latest block.
    LatestBlock(Hash, PeerId),
    /// Request for blocks after the block with `Hash` for the peer with `PeerId`.
    GetBlocksAfter(Hash, PeerId),
    /// The response to `GetBlocksAfter`. Contains the requested blocks and the id of the peer who shared them.
    ShareBlocks(Vec<ValidBlock>, PeerId),
}

/// Transaction data is permanently recorded in files called blocks. Blocks are organized into
/// a linear sequence over time (also known as the block chain).
/// Blocks lifecycle starts from "Pending" state which is represented by `PendingBlock` struct.
#[derive(Clone, Debug, Encode, Decode)]
pub struct PendingBlock {
    /// Unix time (in milliseconds) of block forming by a peer.
    pub timestamp: u128,
    /// array of transactions, which successfully passed validation and consensus step.
    pub transactions: Vec<SignedTransaction>,
}

impl PendingBlock {
    // /// Create a new `PendingBlock` from transactions.
    // pub fn new(
    //     transactions: Vec<AcceptedTransaction>,
    //     key_pair: &KeyPair,
    // ) -> Result<PendingBlock, String> {
    //     Ok(PendingBlock {
    //         timestamp: SystemTime::now()
    //             .duration_since(SystemTime::UNIX_EPOCH)
    //             .expect("Failed to get System Time.")
    //             .as_millis(),
    //         transactions: transactions
    //             .iter()
    //             .cloned()
    //             .map(|transaction| transaction.sign(key_pair))
    //             .collect::<Result<Vec<_>, _>>()?,
    //     })
    // }

    /// Chain block with the existing blockchain.
    pub fn chain(
        self,
        height: u64,
        previous_block_hash: Hash,
        number_of_view_changes: u32,
        invalidated_blocks_hashes: Vec<Hash>,
    ) -> ChainedBlock {
        ChainedBlock {
            transactions: self.transactions,
            header: BlockHeader {
                timestamp: self.timestamp,
                height: height + 1,
                previous_block_hash,
                // TODO: get actual merkle tree hash
                merkle_root_hash: [0u8; 32],
                number_of_view_changes,
                invalidated_blocks_hashes,
            },
        }
    }

    /// Create a new blockchain with current block as a first block.
    pub fn chain_first(self) -> ChainedBlock {
        ChainedBlock {
            transactions: self.transactions,
            header: BlockHeader {
                timestamp: self.timestamp,
                height: 0,
                previous_block_hash: [0u8; 32],
                merkle_root_hash: [0u8; 32],
                number_of_view_changes: 0,
                invalidated_blocks_hashes: Vec::new(),
            },
        }
    }
}

/// When `PendingBlock` chained with a blockchain it becomes `ChainedBlock`
#[derive(Clone, Debug, Encode, Decode)]
pub struct ChainedBlock {
    /// Header
    pub header: BlockHeader,
    /// Array of transactions, which successfully passed validation and consensus step.
    pub transactions: Vec<SignedTransaction>,
}

/// Header of the block. The hash should be taken from its byte representation.
#[derive(Clone, Debug, Encode, Decode)]
pub struct BlockHeader {
    /// Unix time (in milliseconds) of block forming by a peer.
    pub timestamp: u128,
    /// a number of blocks in the chain up to the block.
    pub height: u64,
    /// Hash of a previous block in the chain.
    /// Is an array of zeros for the first block.
    pub previous_block_hash: Hash,
    /// Hash of merkle tree root of the tree of transactions hashes.
    pub merkle_root_hash: Hash,
    /// Number of view changes after the previous block was committed and before this block was committed.
    pub number_of_view_changes: u32,
    /// Hashes of the blocks that were rejected by consensus.
    pub invalidated_blocks_hashes: Vec<Hash>,
}

// impl BlockHeader {
//     /// Calculate hash of the current block header.
//     pub fn hash(&self) -> Hash {
//         crypto::hash(self.into())
//     }
// }

/// After full validation `ChainedBlock` can transform into `ValidBlock`.
#[derive(Clone, Debug, Encode, Decode)]
pub struct ValidBlock {
    /// Header
    pub header: BlockHeader,
    /// Array of transactions.
    pub transactions: Vec<ValidTransaction>,
    /// Signatures of peers which approved this block.
    pub signatures: Signatures,
}

/*
impl ValidBlock {
    /// Commit block to the store.
    //TODO: pass block store and block sender as parameters?
    pub fn commit(self) -> CommittedBlock {
        CommittedBlock {
            header: self.header,
            transactions: self.transactions,
            signatures: self.signatures,
        }
    }

    /// Validate block transactions against current state of the world.
    pub fn validate(self, world_state_view: &WorldStateView) -> ValidBlock {
        let mut world_state_view = world_state_view.clone();
        let mut transactions = Vec::new();
        for transaction in self.transactions {
            match transaction.validate(&mut world_state_view) {
                Ok(transaction) => transactions.push(transaction),
                Err(e) => log::warn!("Transaction validation failed: {}", e),
            }
        }
        //TODO: rebuild merkle tree and reassign `merkle_root_hash`, as tx set may be different.
        ValidBlock {
            header: self.header,
            transactions,
            signatures: self.signatures,
        }
    }

    /// Calculate hash of the current block.
    pub fn hash(&self) -> Hash {
        self.header.hash()
    }

    /// Sign this block and get `ValidBlock`.
    pub fn sign(mut self, key_pair: &KeyPair) -> Result<ValidBlock, String> {
        let signature = Signature::new(key_pair.clone(), &self.hash())?;
        self.signatures.add(signature);
        Ok(self)
    }

    /// Signatures that are verified with the `hash` of this block as `payload`.
    pub fn verified_signatures(&self) -> Vec<Signature> {
        self.signatures.verified(&self.hash())
    }
}
*/

/// When Kura receives `ValidBlock`, the block is stored and
/// then sent to later stage of the pipeline as `CommitedBlock`.
#[derive(Clone, Debug, Encode, Decode)]
pub struct CommittedBlock {
    /// Header
    pub header: BlockHeader,
    /// array of transactions, which successfully passed validation and consensus step.
    pub transactions: Vec<ValidTransaction>,
    /// Signatures of peers which approved this block
    pub signatures: Signatures,
}
/*
impl CommittedBlock {
    /// Calculate hash of the current block.
    /// `CommitedBlock` should have the same hash as `ValidBlock`.
    pub fn hash(&self) -> Hash {
        self.header.hash()
    }
}
*/
