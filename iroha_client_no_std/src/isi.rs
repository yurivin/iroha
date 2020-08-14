//! This module contains enumeration of all legal Iroha Special Instructions `Instruction`
//! and related implementations.

use super::query::IrohaQuery;
// use iroha_derive::Io;
use alloc::boxed::Box;
use alloc::{string::String, vec::Vec};
use parity_scale_codec::{Decode, Encode};
use crate::Identifiable;

pub mod prelude {
    //! Re-exports important traits and types. Meant to be glob imported when using `Iroha`.
    pub use crate::{account::isi::*, asset::isi::*, domain::isi::*, isi::*, peer::isi::*};
}

/// Enumeration of all legal Iroha Special Instructions.
#[derive(Clone, Debug, Encode, Decode)]
#[allow(clippy::large_enum_variant)]
pub enum Instruction {
    /// Variant of instructions related to `Peer`.
    Peer(crate::peer::isi::PeerInstruction),
    /// Instruction variants related to `Domain`.
    Domain(crate::domain::isi::DomainInstruction),
    /// Instruction variants related to `Asset`.
    Asset(crate::asset::isi::AssetInstruction),
    /// Instruction variants related to `Account`.
    Account(crate::account::isi::AccountInstruction),
    /// Instruction variants related to `Permission`.
    Permission(crate::permission::isi::PermissionInstruction),
    /// Instruction variants connected to different Iroha Events.
    Event(crate::event::isi::EventInstruction),
    /// This variant of Iroha Special Instruction composes two other instructions into one, and
    /// executes them both.
    Compose(Box<Instruction>, Box<Instruction>),
    /// This variant of Iroha Special Instruction composes several other instructions into one, and
    /// executes them one by one. If some instruction fails - the whole sequence will fail.
    Sequence(Vec<Instruction>),
    /// This variant of Iroha Special Instruction executes the Iroha Query.
    ExecuteQuery(IrohaQuery),
    /// This variant of Iroha Special Instruction executes the first instruction and if it succeeded
    /// executes the second one, if failed - the third one if presented.
    If(Box<Instruction>, Box<Instruction>, Option<Box<Instruction>>),
    /// This variant of Iroha Special Instructions explicitly returns an error with the given
    /// message.
    Fail(String),
    /// This variant of Iroha Special Instructions sends notifications.
    Notify(String),
}

/// Generic instruction for an addition of an object to the identifiable destination.
pub struct Add<D, O>
    where
        D: Identifiable,
{
    /// Object which should be added.
    pub object: O,
    /// Destination object `Id`.
    pub destination_id: D::Id,
}

impl<D, O> Add<D, O>
    where
        D: Identifiable,
{
    /// Default `Add` constructor.
    pub fn new(object: O, destination_id: D::Id) -> Self {
        Add {
            object,
            destination_id,
        }
    }
}

/// Generic instruction for a removal of an object from the identifiable destination.
pub struct Remove<D, O>
    where
        D: Identifiable,
{
    /// Object which should be removed.
    pub object: O,
    /// Destination object `Id`.
    pub destination_id: D::Id,
}

impl<D, O> Remove<D, O>
    where
        D: Identifiable,
{
    /// Default `Remove` constructor.
    pub fn new(object: O, destination_id: D::Id) -> Self {
        Remove {
            object,
            destination_id,
        }
    }
}

/// Generic instruction for a registration of an object to the identifiable destination.
pub struct Register<D, O>
    where
        D: Identifiable,
{
    /// Object which should be registered.
    pub object: O,
    /// Destination object `Id`.
    pub destination_id: D::Id,
}

impl<D, O> Register<D, O>
    where
        D: Identifiable,
{
    /// Default `Register` constructor.
    pub fn new(object: O, destination_id: D::Id) -> Self {
        Register {
            object,
            destination_id,
        }
    }
}

/// Generic instruction for a mint of an object to the identifiable destination.
pub struct Mint<D, O>
    where
        D: Identifiable,
{
    /// Object which should be minted.
    pub object: O,
    /// Destination object `Id`.
    pub destination_id: D::Id,
}

impl<D, O> Mint<D, O>
    where
        D: Identifiable,
{
    /// Default `Mint` constructor.
    pub fn new(object: O, destination_id: D::Id) -> Self {
        Mint {
            object,
            destination_id,
        }
    }
}

/// Generic instruction for a demint of an object to the identifiable destination.
pub struct Demint<D, O>
    where
        D: Identifiable,
{
    /// Object which should be deminted.
    pub object: O,
    /// Destination object `Id`.
    pub destination_id: D::Id,
}

impl<D, O> Demint<D, O>
    where
        D: Identifiable,
{
    /// Default `Demint` constructor.
    pub fn new(object: O, destination_id: D::Id) -> Self {
        Demint {
            object,
            destination_id,
        }
    }
}

/// Generic instruction for a transfer of an object from the identifiable source to the identifiable destination.
pub struct Transfer<Src: Identifiable, Obj, Dst: Identifiable> {
    /// Source object `Id`.
    pub source_id: Src::Id,
    /// Object which should be transfered.
    pub object: Obj,
    /// Destination object `Id`.
    pub destination_id: Dst::Id,
}

impl<Src: Identifiable, Obj, Dst: Identifiable> Transfer<Src, Obj, Dst> {
    /// Default `Transfer` constructor.
    pub fn new(source_id: Src::Id, object: Obj, destination_id: Dst::Id) -> Self {
        Transfer {
            source_id,
            object,
            destination_id,
        }
    }
}
