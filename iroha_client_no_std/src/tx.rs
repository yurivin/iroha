//! This module contains Transaction related functionality of the Iroha.
//!
//! `RequestedTransaction` is the start of the Transaction lifecycle.

use crate::crypto::Signature;
use crate::prelude::*;
// use iroha_derive::Io;
use alloc::{string::String, vec::Vec};
use parity_scale_codec::{Decode, Encode};

/// This structure represents transaction in non-trusted form.
///
/// `Iroha` and its' clients use `RequestedTransaction` to send transactions via network.
/// Direct usage in business logic is strongly prohibited. Before any interactions
/// `accept`.
#[derive(Clone, Debug, Encode, Decode)]
pub struct RequestedTransaction {
    pub payload: Payload,
    pub signatures: Vec<Signature>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct Payload {
    /// Account ID of transaction creator.
    pub account_id: <Account as Identifiable>::Id,
    /// An ordered set of instructions.
    pub instructions: Vec<Instruction>,
    /// Time of creation (unix time, in milliseconds).
    pub creation_time: u64,
    /// The transaction will be dropped after this time if it is still in a `Queue`.
    pub time_to_live_ms: u64,
}

impl RequestedTransaction {
    /// Default `RequestedTransaction` constructor.
    pub fn new(
        instructions: Vec<Instruction>,
        account_id: <Account as Identifiable>::Id,
        proposed_ttl_ms: u64,
        creation_time: u64,
    ) -> RequestedTransaction {
        RequestedTransaction {
            payload: Payload {
                instructions,
                account_id,
                creation_time, // Utc::now().timestamp_millis() as u64,
                time_to_live_ms: proposed_ttl_ms,
            },
            signatures: Vec::new(),
        }
    }

    // /// Transaction acceptance will check that transaction signatures are valid and move state one
    // /// step forward.
    // ///
    // /// Returns `Ok(AcceptedTransaction)` if succeeded and `Err(String)` if failed.
    // pub fn accept(self) -> Result<AcceptedTransaction, String> {
    //     for signature in &self.signatures {
    //         if let Err(e) = signature.verify(&self.payload.encode()) {
    //             return Err(format!("Failed to verify signatures: {}", e));
    //         }
    //     }
    //     Ok(AcceptedTransaction {
    //         payload: self.payload,
    //         signatures: self.signatures,
    //     })
    // }
}

/// An ordered set of instructions, which is applied to the ledger atomically.
///
/// Transactions received by `Iroha` from external resources (clients, peers, etc.)
/// go through several steps before will be added to the blockchain and stored.
/// Starting in form of `RequestedTransaction` transaction it changes state based on interactions
/// with `Iroha` subsystems.
#[derive(Clone, Debug, Encode, Decode)]
pub struct AcceptedTransaction {
    payload: Payload,
    signatures: Vec<Signature>,
}

impl AcceptedTransaction {
    // /// Sign transaction with the provided key pair.
    // ///
    // /// Returns `Ok(SignedTransaction)` if succeeded and `Err(String)` if failed.
    // pub fn sign(self, key_pair: &KeyPair) -> Result<SignedTransaction, String> {
    //     let mut signatures = self.signatures.clone();
    //     signatures.push(Signature::new(key_pair.clone(), &self.payload.encode())?);
    //     Ok(SignedTransaction {
    //         payload: self.payload,
    //         signatures,
    //     })
    // }

    // /// Calculate transaction `Hash`.
    // pub fn hash(&self) -> Hash {
    //     use ursa::blake2::{
    //         digest::{Input, VariableOutput},
    //         VarBlake2b,
    //     };
    //     let bytes: Vec<u8> = self.payload.clone().encode();
    //     let vec_hash = VarBlake2b::new(32)
    //         .expect("Failed to initialize variable size hash")
    //         .chain(bytes)
    //         .vec_result();
    //     let mut hash = [0; 32];
    //     hash.copy_from_slice(&vec_hash);
    //     hash
    // }
}

/// `SignedTransaction` represents transaction with signatures accumulated from Peer/Peers.
#[derive(Clone, Debug, Encode, Decode)]
pub struct SignedTransaction {
    payload: Payload,
    signatures: Vec<Signature>,
}

impl SignedTransaction {
    /// Add additional Signatures.
    pub fn sign(self, signatures: Vec<Signature>) -> Result<SignedTransaction, String> {
        Ok(SignedTransaction {
            payload: self.payload,
            signatures: vec![self.signatures, signatures]
                .into_iter()
                .flatten()
                .collect(),
        })
    }

    // /// Calculate transaction `Hash`.
    // pub fn hash(&self) -> Hash {
    //     use ursa::blake2::{
    //         digest::{Input, VariableOutput},
    //         VarBlake2b,
    //     };
    //     let bytes: Vec<u8> = self.encode();
    //     // let bytes: Vec<u8> = self.into();
    //     let vec_hash = VarBlake2b::new(32)
    //         .expect("Failed to initialize variable size hash")
    //         .chain(bytes)
    //         .vec_result();
    //     let mut hash = [0; 32];
    //     hash.copy_from_slice(&vec_hash);
    //     hash
    // }
}

impl From<&SignedTransaction> for RequestedTransaction {
    fn from(transaction: &SignedTransaction) -> RequestedTransaction {
        let transaction = transaction.clone();
        RequestedTransaction::from(transaction)
    }
}

impl From<SignedTransaction> for RequestedTransaction {
    fn from(transaction: SignedTransaction) -> RequestedTransaction {
        RequestedTransaction {
            payload: transaction.payload,
            signatures: transaction.signatures,
        }
    }
}

/// `ValidTransaction` represents trustfull Transaction state.
#[derive(Clone, Debug, Encode, Decode)]
pub struct ValidTransaction {
    pub payload: Payload,
    pub signatures: Vec<Signature>,
}
