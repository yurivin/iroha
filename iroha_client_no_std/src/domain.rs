//! This module contains `Domain` structure and related implementations.

use crate::prelude::*;
use alloc::{collections::BTreeMap, string::String};
// use iroha_derive::*;
use parity_scale_codec::{Decode, Encode};
type Name = String;
/// Named group of `Account` and `Asset` entities.
#[derive(Debug, Clone, Encode, Decode)]
pub struct Domain {
    /// Domain name, for example company name.
    pub name: Name,
    /// Accounts of the domain.
    pub accounts: BTreeMap<<Account as Identifiable>::Id, Account>,
    /// Assets of the domain.
    pub asset_definitions: BTreeMap<<AssetDefinition as Identifiable>::Id, AssetDefinition>,
}

impl Domain {
    /// Creates new detached `Domain`.
    ///
    /// Should be used for creation of a new `Domain` or while making queries.
    pub fn new(name: Name) -> Self {
        Domain {
            name,
            accounts: BTreeMap::new(),
            asset_definitions: BTreeMap::new(),
        }
    }
}

impl Identifiable for Domain {
    type Id = Name;
}

/// Iroha Special Instructions module provides `DomainInstruction` enum with all legal types of
/// Domain related instructions as variants, implementations of generic Iroha Special Instructions
/// and the `From/Into` implementations to convert `DomainInstruction` variants into generic ISI.
pub mod isi {
    use super::*;

    /// Enumeration of all legal Domain related Instructions.
    #[derive(Clone, Debug, Encode, Decode)]
    pub enum DomainInstruction {
        /// Variant of the generic `Register` instruction for `Account` --> `Domain`.
        RegisterAccount(Name, Account),
        /// Variant of the generic `Register` instruction for `AssetDefinition` --> `Domain`.
        RegisterAsset(Name, AssetDefinition),
    }
}

/// Query module provides `IrohaQuery` Domain related implementations.
pub mod query {
    use super::*;
    use crate::query::IrohaQuery;
    use parity_scale_codec::{Decode, Encode};

    /// Get information related to the domain with a specified `domain_name`.
    #[derive(Clone, Debug, Encode, Decode)]
    pub struct GetDomain {
        /// Identification of an domain to find information about.
        pub domain_name: <Domain as Identifiable>::Id,
    }

    /// Result of the `GetDomain` execution.
    #[derive(Clone, Debug, Encode, Decode)]
    pub struct GetDomainResult {
        /// Domain information.
        pub domain: Domain,
    }

    impl GetDomain {
        /// Build a `GetDomain` query in the form of a `QueryRequest`.
        pub fn build_request(domain_name: <Domain as Identifiable>::Id) -> QueryRequest {
            let query = GetDomain { domain_name };
            QueryRequest {
                timestamp: "".into(),
                signature: Option::None,
                query: IrohaQuery::GetDomain(query),
            }
        }
    }
}
