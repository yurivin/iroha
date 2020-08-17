//! This module contains functionality related to `Bridge`.

use crate::alloc::borrow::ToOwned;
use crate::alloc::string::ToString;
use crate::asset::Bytes;
use crate::crypto::PublicKey;
use crate::isi::prelude::*;
use crate::isi::*;
use crate::prelude::*;
use alloc::{boxed::Box, string::String, vec::Vec};
use parity_scale_codec::{Decode, Encode};

const BRIDGE_ACCOUNT_NAME: &str = "bridge";
const BRIDGE_ASSET_BRIDGE_DEFINITION_PARAMETER_KEY: &str = "bridge_definition";

/// Enumeration of all supported bridge kinds (types). Each variant represents some communication
/// protocol between blockchains which can be used within Iroha.
#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum BridgeKind {
    /// XClaim-like protocol.
    IClaim,
}

/// Identification of a Bridge definition. Consists of Bridge's name.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Encode, Decode)]
pub struct BridgeDefinitionId {
    /// Bridge's name.
    pub name: String,
}

impl BridgeDefinitionId {
    /// Default Bridge definition identifier constructor.
    pub fn new(name: &str) -> Self {
        BridgeDefinitionId {
            name: name.to_owned(),
        }
    }
}

/// A data required for `Bridge` entity initialization.
#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, Hash)]
pub struct BridgeDefinition {
    /// An Identification of the `BridgeDefinition`.
    pub id: <BridgeDefinition as Identifiable>::Id,
    /// Bridge's kind (type).
    pub kind: BridgeKind,
    /// Bridge owner's account Identification. Only this account will be able to manipulate the bridge.
    pub owner_account_id: <Account as Identifiable>::Id,
}

impl Identifiable for BridgeDefinition {
    type Id = BridgeDefinitionId;
}

/// Identification of a Bridge. Consists of Bridge's definition Identification.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Encode, Decode)]
pub struct BridgeId {
    /// Entity Identification.
    definition_id: <BridgeDefinition as Identifiable>::Id,
}

impl BridgeId {
    /// Default Bridge identifier constructor.
    pub fn new(name: &str) -> Self {
        BridgeId {
            definition_id: BridgeDefinitionId::new(name),
        }
    }

    /// Bridge name.
    pub fn name(&self) -> &str {
        &self.definition_id.name
    }
}

/// An entity used for performing operations between Iroha and third-party blockchain.
#[derive(Debug, Clone)]
pub struct Bridge {
    /// Component Identification.
    id: <Bridge as Identifiable>::Id,
    /// Bridge's account ID.
    account_id: <Account as Identifiable>::Id,
}

impl Bridge {
    /// Default `Bridge` entity constructor.
    pub fn new(
        id: <Bridge as Identifiable>::Id,
        account_id: <Account as Identifiable>::Id,
    ) -> Self {
        Bridge { id, account_id }
    }

    /// A helper function for returning Bridge's name.
    pub fn name(&self) -> &str {
        &self.id.definition_id.name
    }
}

impl Identifiable for Bridge {
    type Id = BridgeId;
}

/// An entity used for storing data of any third-party transaction.
#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug, Encode, Decode)]
pub struct ExternalTransaction {
    /// External transaction identifier. Not always can be calculated from the `payload`.
    pub hash: String,
    /// External transaction payload.
    pub payload: Bytes,
}

#[inline]
pub fn bridges_asset_definition_id() -> <AssetDefinition as Identifiable>::Id {
    AssetDefinitionId::new("bridges_asset", "bridge")
}

#[inline]
pub fn bridge_asset_definition_id() -> <AssetDefinition as Identifiable>::Id {
    AssetDefinitionId::new("bridge_asset", "bridge")
}

#[inline]
pub fn bridge_external_assets_asset_definition_id() -> <AssetDefinition as Identifiable>::Id {
    AssetDefinitionId::new("bridge_external_assets_asset", "bridge")
}

#[inline]
pub fn bridge_incoming_external_transactions_asset_definition_id(
) -> <AssetDefinition as Identifiable>::Id {
    AssetDefinitionId::new("bridge_incoming_external_transactions_asset", "bridge")
}

#[inline]
pub fn bridge_outgoing_external_transactions_asset_definition_id(
) -> <AssetDefinition as Identifiable>::Id {
    AssetDefinitionId::new("bridge_outgoing_external_transactions_asset", "bridge")
}

/// This module provides structures for working with external assets.
///
/// # Note
/// `ExternalAsset` is incompatible with Iroha `Asset`.
pub mod asset {
    use super::*;

    /// External asset Identifier.
    pub type Id = String;

    /// A data required for `ExternalAsset` entity initialization.
    #[cfg_attr(test, derive(PartialEq, Eq))]
    #[derive(Debug, Clone, Encode, Decode)]
    pub struct ExternalAsset {
        /// Component Identification.
        pub bridge_id: <Bridge as Identifiable>::Id,
        /// External asset's name in the Iroha.
        pub name: String,
        /// External asset ID.
        pub id: Id,
        /// The number of digits that come after the decimal place.
        /// Used in a value representation.
        pub decimals: u8,
    }

    impl Identifiable for ExternalAsset {
        type Id = Id;
    }
}

/// Iroha Special Instructions module provides helper-methods for `Peer` for registering bridges,
/// bridge clients and external assets.
pub mod isi {
    use super::*;
    use crate::account::query::*;
    use crate::bridge::asset::*;
    use crate::crypto::PublicKey;

    /// Constructor of Iroha Special Instruction for bridge registration.
    pub fn register_bridge(
        peer_id: <Peer as Identifiable>::Id,
        bridge_definition: &BridgeDefinition,
    ) -> Instruction {
        let domain = Domain::new(bridge_definition.id.name.clone());
        let account = Account::new(BRIDGE_ACCOUNT_NAME, &domain.name);
        Instruction::If(
            Box::new(Instruction::ExecuteQuery(IrohaQuery::GetAccount(
                GetAccount {
                    account_id: bridge_definition.owner_account_id.clone(),
                },
            ))),
            Box::new(Instruction::Sequence(vec![
                Add {
                    object: domain.clone(),
                    destination_id: peer_id,
                }
                .into(),
                Register {
                    object: account.clone(),
                    destination_id: domain.name,
                }
                .into(),
                Mint {
                    object: (
                        BRIDGE_ASSET_BRIDGE_DEFINITION_PARAMETER_KEY.to_string(),
                        bridge_definition.encode(),
                    ),
                    destination_id: AssetId {
                        definition_id: bridge_asset_definition_id(),
                        account_id: account.id,
                    },
                }
                .into(),
                Mint {
                    object: (
                        bridge_definition.id.name.clone(),
                        bridge_definition.encode(),
                    ),
                    destination_id: AssetId {
                        definition_id: bridges_asset_definition_id(),
                        account_id: bridge_definition.owner_account_id.clone(),
                    },
                }
                .into(),
                // TODO: add incoming transfer event listener
            ])),
            Some(Box::new(Instruction::Fail(
                "Account not found.".to_string(),
            ))),
        )
    }

    /// Constructor of Iroha Special Instruction for external asset registration.
    pub fn register_external_asset(external_asset: &ExternalAsset) -> Instruction {
        let domain_id = &external_asset.bridge_id.definition_id.name;
        let account_id = AccountId::new(BRIDGE_ACCOUNT_NAME, domain_id);
        let asset_definition = AssetDefinition::new(AssetDefinitionId::new(
            &external_asset.id,
            &external_asset.bridge_id.definition_id.name,
        ));
        Instruction::Sequence(vec![
            Register {
                object: asset_definition,
                destination_id: domain_id.clone(),
            }
            .into(),
            Mint {
                object: (external_asset.id.clone(), external_asset.encode()),
                destination_id: AssetId {
                    definition_id: bridge_external_assets_asset_definition_id(),
                    account_id,
                },
            }
            .into(),
        ])
    }

    /// Constructor of Iroha Special Instruction for adding bridge client.
    pub fn add_client(
        bridge_definition_id: &<BridgeDefinition as Identifiable>::Id,
        client_public_key: PublicKey,
    ) -> Instruction {
        let domain_id = &bridge_definition_id.name;
        let account_id = AccountId::new(BRIDGE_ACCOUNT_NAME, domain_id);
        Add {
            object: client_public_key,
            destination_id: account_id,
        }
        .into()
    }

    /// Constructor of Iroha Special Instruction for removing bridge client.
    pub fn remove_client(
        bridge_definition_id: &<BridgeDefinition as Identifiable>::Id,
        client_public_key: PublicKey,
    ) -> Instruction {
        let domain_id = &bridge_definition_id.name;
        let account_id = AccountId::new(BRIDGE_ACCOUNT_NAME, domain_id);
        Remove {
            object: client_public_key,
            destination_id: account_id,
        }
        .into()
    }

    /// Constructor of Iroha Special Instruction for registering incoming transfer and minting
    /// the external asset to the recipient.
    pub fn handle_incoming_transfer(
        bridge_definition_id: &<BridgeDefinition as Identifiable>::Id,
        asset_defintion_id: &<AssetDefinition as Identifiable>::Id,
        quantity: u32,
        big_quantity: u128,
        recipient: <Account as Identifiable>::Id,
        transaction: &ExternalTransaction,
    ) -> Instruction {
        let domain_id = &bridge_definition_id.name;
        let account_id = AccountId::new(BRIDGE_ACCOUNT_NAME, domain_id);
        let asset_id = AssetId {
            definition_id: asset_defintion_id.clone(),
            account_id: recipient,
        };
        Instruction::Sequence(vec![
            Mint::new(quantity, asset_id.clone()).into(),
            Mint::new(big_quantity, asset_id).into(),
            Mint::new(
                (transaction.hash.clone(), transaction.encode()),
                AssetId {
                    definition_id: bridge_incoming_external_transactions_asset_definition_id(),
                    account_id,
                },
            )
            .into(),
        ])
    }

    /// Constructor of Iroha Special Instruction for registering outgoing transfer and deminting
    /// received asset.
    pub fn handle_outgoing_transfer(
        bridge_definition_id: &<BridgeDefinition as Identifiable>::Id,
        asset_definition_id: &<AssetDefinition as Identifiable>::Id,
        quantity: u32,
        big_quantity: u128,
        transaction: &ExternalTransaction,
    ) -> Instruction {
        let domain_id = &bridge_definition_id.name;
        let account_id = AccountId::new(BRIDGE_ACCOUNT_NAME, domain_id);
        let asset_id = AssetId {
            definition_id: asset_definition_id.clone(),
            account_id: account_id.clone(),
        };
        Instruction::Sequence(vec![
            Demint::new(quantity, asset_id.clone()).into(),
            Demint::new(big_quantity, asset_id).into(),
            Mint::new(
                (transaction.hash.clone(), transaction.encode()),
                AssetId {
                    definition_id: bridge_outgoing_external_transactions_asset_definition_id(),
                    account_id,
                },
            )
            .into(),
        ])
    }
}

/// Query module provides functions for constructing bridge-related queries
/// and decoding the query results.
pub mod query {
    use super::asset::*;
    use super::*;
    use crate::crypto::PublicKey;

    /// Constructor of Iroha Query for retrieving list of all registered bridges.
    pub fn query_bridges_list(bridge_owner_id: <Account as Identifiable>::Id) -> IrohaQuery {
        crate::asset::query::GetAccountAssets::build_request(bridge_owner_id).query
    }

    /// A helper function for decoding a list of bridge definitions from the query result.
    ///
    /// Each `BridgeDefinition` is encoded and stored in the bridges asset
    /// (`bridges_asset_definition_id`) store indexed by a name of the bridge. The given query
    /// result may not contain the above values, so this function can fail, returning `None`.
    pub fn decode_bridges_list<'a>(
        query_result: &'a QueryResult,
    ) -> Option<impl Iterator<Item = BridgeDefinition> + 'a> {
        let account_assets_result = match query_result {
            QueryResult::GetAccountAssets(v) => v,
            _ => return None,
        };
        account_assets_result
            .assets
            .iter()
            .find(|asset| asset.id.definition_id == bridges_asset_definition_id())
            .map(|asset| {
                asset
                    .store
                    .values()
                    .filter_map(|data| BridgeDefinition::decode(&mut data.as_slice()).ok())
            })
    }

    /// Constructor of Iroha Query for retrieving information about the bridge.
    pub fn query_bridge(bridge_id: <Bridge as Identifiable>::Id) -> IrohaQuery {
        crate::account::query::GetAccount::build_request(AccountId::new(
            BRIDGE_ACCOUNT_NAME,
            bridge_id.name(),
        ))
        .query
    }

    /// A helper function for decoding bridge definition from the query result.
    ///
    /// The `BridgeDefinition` is encoded and stored in the bridge asset
    /// (`bridge_asset_definition_id`) store under the
    /// `BRIDGE_ASSET_BRIDGE_DEFINITION_PARAMETER_KEY` key. The given query result may not
    /// contain the above values, so this function can fail, returning `None`.
    pub fn decode_bridge_definition(query_result: &QueryResult) -> Option<BridgeDefinition> {
        let account_result = match query_result {
            QueryResult::GetAccount(v) => v,
            _ => return None,
        };
        account_result
            .account
            .assets
            .iter()
            .find(|(id, _)| id.definition_id == bridge_asset_definition_id())
            .and_then(|(_, asset)| {
                asset
                    .store
                    .get(BRIDGE_ASSET_BRIDGE_DEFINITION_PARAMETER_KEY)
                    .and_then(|data| BridgeDefinition::decode(&mut data.as_slice()).ok())
            })
    }

    /// A helper function for decoding information about external asset from the query result.
    ///
    /// Each `ExternalAsset` is encoded and stored in the bridge external assets asset
    /// (`bridge_external_assets_asset_definition_id`) store and indexed by a name of the asset.
    /// The given query result may not contain the above values, so this function can fail,
    /// returning `None`.
    pub fn decode_external_asset(
        query_result: &QueryResult,
        asset_name: &str,
    ) -> Option<ExternalAsset> {
        let account_result = match query_result {
            QueryResult::GetAccount(v) => v,
            _ => return None,
        };
        account_result
            .account
            .assets
            .iter()
            .find(|(id, _)| id.definition_id == bridge_external_assets_asset_definition_id())
            .and_then(|(_, asset)| {
                asset
                    .store
                    .get(asset_name)
                    .cloned()
                    .and_then(|data| ExternalAsset::decode(&mut data.as_slice()).ok())
            })
    }

    /// A helper function for decoding information about external assets from the query result.
    ///
    /// Each `ExternalAsset` is encoded and stored in the bridge external assets asset
    /// (`bridge_external_assets_asset_definition_id`) store and indexed by a name of the asset.
    /// The given query result may not contain the above values, so this function can fail,
    /// returning `None`.
    pub fn decode_external_assets<'a>(
        query_result: &'a QueryResult,
    ) -> Option<impl Iterator<Item = ExternalAsset> + 'a> {
        let account_result = match query_result {
            QueryResult::GetAccount(v) => v,
            _ => return None,
        };
        account_result
            .account
            .assets
            .iter()
            .find(|(id, _)| id.definition_id == bridge_external_assets_asset_definition_id())
            .map(|(_, asset)| {
                asset
                    .store
                    .values()
                    .filter_map(|data| ExternalAsset::decode(&mut data.as_slice()).ok())
            })
    }

    /// A helper function for retrieving information about bridge clients.
    pub fn get_clients(query_result: &QueryResult) -> Option<&Vec<PublicKey>> {
        let account_result = match query_result {
            QueryResult::GetAccount(v) => v,
            _ => return None,
        };
        Some(&account_result.account.signatories)
    }

    fn decode_external_transactions<'a>(
        query_result: &'a QueryResult,
        is_incoming: bool,
    ) -> Option<impl Iterator<Item = ExternalTransaction> + 'a> {
        let account_result = match query_result {
            QueryResult::GetAccount(v) => v,
            _ => return None,
        };
        account_result
            .account
            .assets
            .iter()
            .find(|(id, _)| {
                let asset_definition_id = if is_incoming {
                    bridge_incoming_external_transactions_asset_definition_id()
                } else {
                    bridge_outgoing_external_transactions_asset_definition_id()
                };
                id.definition_id == asset_definition_id
            })
            .map(|(_, asset)| {
                asset
                    .store
                    .values()
                    .filter_map(|data| ExternalTransaction::decode(&mut data.as_slice()).ok())
            })
    }

    /// A helper function for decoding information about incoming external transactions
    /// from the query result.
    ///
    /// Each `ExternalTransaction` is encoded and stored in the bridge external assets asset
    /// (`bridge_incoming_external_transactions_asset_definition_id`) store and indexed by a
    /// transaction hash. The given query result may not contain the above values, so this
    /// function can fail, returning `None`.
    pub fn decode_incoming_external_transactions<'a>(
        query_result: &'a QueryResult,
    ) -> Option<impl Iterator<Item = ExternalTransaction> + 'a> {
        decode_external_transactions(query_result, true)
    }

    /// A helper function for decoding information about outgoing external transactions
    /// from the query result.
    ///
    /// Each `ExternalTransaction` is encoded and stored in the bridge external assets asset
    /// (`bridge_outgoing_external_transactions_asset_definition_id`) store and indexed by a
    /// transaction hash. The given query result may not contain the above values, so this
    /// function can fail, returning `None`.
    pub fn decode_outgoing_external_transactions<'a>(
        query_result: &'a QueryResult,
    ) -> Option<impl Iterator<Item = ExternalTransaction> + 'a> {
        decode_external_transactions(query_result, false)
    }
}

impl From<Transfer<Account, Asset, Account>> for Instruction {
    fn from(instruction: Transfer<Account, Asset, Account>) -> Self {
        Instruction::Account(AccountInstruction::TransferAsset(
            instruction.source_id,
            instruction.destination_id,
            instruction.object,
        ))
    }
}

impl From<Mint<Asset, u32>> for Instruction {
    fn from(instruction: Mint<Asset, u32>) -> Self {
        Instruction::Asset(AssetInstruction::MintAsset(
            instruction.object,
            instruction.destination_id,
        ))
    }
}
impl From<Mint<Asset, u128>> for Instruction {
    fn from(instruction: Mint<Asset, u128>) -> Self {
        Instruction::Asset(AssetInstruction::MintBigAsset(
            instruction.object,
            instruction.destination_id,
        ))
    }
}
impl From<Mint<Asset, (String, Bytes)>> for Instruction {
    fn from(instruction: Mint<Asset, (String, Bytes)>) -> Self {
        Instruction::Asset(AssetInstruction::MintParameterAsset(
            instruction.object,
            instruction.destination_id,
        ))
    }
}
impl From<Demint<Asset, u32>> for Instruction {
    fn from(instruction: Demint<Asset, u32>) -> Self {
        Instruction::Asset(AssetInstruction::DemintAsset(
            instruction.object,
            instruction.destination_id,
        ))
    }
}
impl From<Demint<Asset, u128>> for Instruction {
    fn from(instruction: Demint<Asset, u128>) -> Self {
        Instruction::Asset(AssetInstruction::DemintBigAsset(
            instruction.object,
            instruction.destination_id,
        ))
    }
}
impl From<Demint<Asset, String>> for Instruction {
    fn from(instruction: Demint<Asset, String>) -> Self {
        Instruction::Asset(AssetInstruction::DemintParameterAsset(
            instruction.object,
            instruction.destination_id,
        ))
    }
}

impl From<Register<Domain, Account>> for Instruction {
    fn from(instruction: Register<Domain, Account>) -> Self {
        Instruction::Domain(DomainInstruction::RegisterAccount(
            instruction.destination_id,
            instruction.object,
        ))
    }
}
impl From<Register<Domain, AssetDefinition>> for Instruction {
    fn from(instruction: Register<Domain, AssetDefinition>) -> Self {
        Instruction::Domain(DomainInstruction::RegisterAsset(
            instruction.destination_id,
            instruction.object,
        ))
    }
}

impl From<Add<Peer, Domain>> for Instruction {
    fn from(add_instruction: Add<Peer, Domain>) -> Self {
        Instruction::Peer(PeerInstruction::AddDomain(
            add_instruction.object.name,
            add_instruction.destination_id,
        ))
    }
}
impl From<Add<Account, PublicKey>> for Instruction {
    fn from(instruction: Add<Account, PublicKey>) -> Self {
        Instruction::Account(AccountInstruction::AddSignatory(
            instruction.destination_id,
            instruction.object,
        ))
    }
}
impl From<Remove<Account, PublicKey>> for Instruction {
    fn from(instruction: Remove<Account, PublicKey>) -> Self {
        Instruction::Account(AccountInstruction::RemoveSignatory(
            instruction.destination_id,
            instruction.object,
        ))
    }
}
