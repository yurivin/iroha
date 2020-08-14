#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

pub mod client;
// pub mod config;
pub mod crypto;
pub mod maintenance;
pub mod torii;
// TODO(vmarkushin): update documentation for the client-side entities (IR-848).
pub mod account;
pub mod asset;
pub mod block;
pub mod domain;
pub mod event;
pub mod isi;
pub mod peer;
pub mod permission;
pub mod query;
pub mod tx;
pub mod bridge;

pub mod prelude {
    #[doc(inline)]
    pub use crate::{
        account::{Account, Id as AccountId},
        asset::{Asset, AssetDefinition, AssetDefinitionId, AssetId},
        domain::Domain,
        isi::Instruction,
        peer::{Peer, PeerId},
        query::{IrohaQuery, QueryRequest, QueryResult},
        tx::{AcceptedTransaction, RequestedTransaction, SignedTransaction},
        Identifiable,
    };
}

/// This trait marks entity that implement it as identifiable with an `Id` type to find them by.
pub trait Identifiable {
    /// Defines the type of entity's identification.
    type Id;
}
