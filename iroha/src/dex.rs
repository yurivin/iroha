//! This module contains functionality related to `DEX`.

use crate::prelude::*;
use iroha_derive::Io;
use parity_scale_codec::{Decode, Encode};

const DEX_ACCOUNT_NAME: &str = "dex";
const DEX_ASSET_DEX_DEFINITION_PARAMETER_KEY: &str = "dex_definition";

/// Identification of a DEX definition.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Encode, Decode)]
pub struct DEXDefinitionId {
    /// Name of given DEX.
    pub name: String,
}

impl DEXDefinitionId {
    /// Default DEX definition identifier constructor.
    pub fn new(name: &str) -> Self {
        DEXDefinitionId {
            name: name.to_owned(),
        }
    }
}

/// A data required for `DEX` entity initialization.
#[derive(Encode, Decode, PartialEq, Eq, Hash, Clone, Debug, Io)]
pub struct DEXDefinition {
    /// An identification of the `DEXDefinition`.
    pub id: <DEXDefinition as Identifiable>::Id,
    /// DEX owner's account Identification. Only this account will be able to manipulate the DEX.
    pub owner_account_id: <Account as Identifiable>::Id,
}

impl Identifiable for DEXDefinition {
    type Id = DEXDefinitionId;
}

#[inline]
fn dex_asset_definition_id() -> <AssetDefinition as Identifiable>::Id {
    AssetDefinitionId::new("dex_asset", "dex")
}

#[inline]
fn dex_collection_asset_definition_id() -> <AssetDefinition as Identifiable>::Id {
    AssetDefinitionId::new("dex_collection_asset", "dex")
}

/// Iroha Special Instructions module provides helper-methods for `Peer` for operating DEX,
/// Token Pairs and Liquidity Sources.
pub mod isi {
    use super::*;
    use crate::account::query::*;

    /// Constructor of Iroha Special Instruction for DEX initialization.
    /// Multiple DEX can be initialized with different names in their definition.
    pub fn initialize_dex(
        peer_id: <Peer as Identifiable>::Id,
        dex_definition: &DEXDefinition,
    ) -> Instruction {
        let domain = Domain::new(dex_definition.id.name.clone());
        let account = Account::new(DEX_ACCOUNT_NAME, &domain.name);
        Instruction::If(
            Box::new(Instruction::ExecuteQuery(IrohaQuery::GetAccount(
                GetAccount {
                    account_id: dex_definition.owner_account_id.clone(),
                },
            ))),
            Box::new(Instruction::Sequence(vec![
                // Create domain for given DEX definition.
                Add {
                    object: domain.clone(),
                    destination_id: peer_id,
                }
                .into(),
                // Register account for given DEX definition.
                Register {
                    object: account.clone(),
                    destination_id: domain.name,
                }
                .into(),
                // Mint new asset for given DEX definition.
                Mint {
                    object: (
                        DEX_ASSET_DEX_DEFINITION_PARAMETER_KEY.to_string(),
                        dex_definition.encode(),
                    ),
                    destination_id: AssetId {
                        definition_id: dex_asset_definition_id(),
                        account_id: account.id,
                    },
                }
                .into(),
                // Mint new asset for collection of DEX definitions, or add new definition
                // to collection if exists.
                Mint {
                    object: (dex_definition.id.name.clone(), dex_definition.encode()),
                    destination_id: AssetId {
                        definition_id: dex_collection_asset_definition_id(),
                        account_id: dex_definition.owner_account_id.clone(),
                    },
                }
                .into(),
            ])),
            Some(Box::new(Instruction::Fail(
                "Account not found.".to_string(),
            ))),
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::dex::query::*;
        use crate::peer::PeerId;
        use crate::permission::{permission_asset_definition_id, Permission};
        use std::collections::BTreeMap;

        const DEX_NAME: &str = "Default";

        struct TestKit {
            world_state_view: WorldStateView,
            root_account_id: <Account as Identifiable>::Id,
        }

        impl TestKit {
            pub fn new() -> Self {
                let domain_name = "Company".to_string();
                let key_pair = KeyPair::generate().expect("Failed to generate KeyPair.");
                let mut asset_definitions = BTreeMap::new();
                let asset_definition_id = permission_asset_definition_id();
                asset_definitions.insert(
                    asset_definition_id.clone(),
                    AssetDefinition::new(asset_definition_id.clone()),
                );
                let account_id = AccountId::new("root", &domain_name);
                let asset_id = AssetId {
                    definition_id: asset_definition_id,
                    account_id: account_id.clone(),
                };
                let asset = Asset::with_permission(asset_id.clone(), Permission::Anything);
                let mut account = Account::with_signatory(
                    &account_id.name,
                    &account_id.domain_name,
                    key_pair.public_key.clone(),
                );
                account.assets.insert(asset_id.clone(), asset);
                let mut accounts = BTreeMap::new();
                accounts.insert(account_id.clone(), account);
                let domain = Domain {
                    name: domain_name.clone(),
                    accounts,
                    asset_definitions,
                };
                let dex_domain_name = DEX_ACCOUNT_NAME.to_string();
                let mut dex_asset_definitions = BTreeMap::new();
                let asset_definition_ids = [
                    dex_asset_definition_id(),
                    dex_collection_asset_definition_id(),
                ];
                for asset_definition_id in &asset_definition_ids {
                    dex_asset_definitions.insert(
                        asset_definition_id.clone(),
                        AssetDefinition::new(asset_definition_id.clone()),
                    );
                }
                let dex_domain = Domain {
                    name: dex_domain_name.clone(),
                    accounts: BTreeMap::new(),
                    asset_definitions: dex_asset_definitions,
                };
                let mut domains = BTreeMap::new();
                domains.insert(domain_name.clone(), domain);
                domains.insert(dex_domain_name.clone(), dex_domain);
                let address = "127.0.0.1:8080".to_string();
                let world_state_view = WorldStateView::new(Peer::with_domains(
                    PeerId {
                        address: address.clone(),
                        public_key: key_pair.public_key,
                    },
                    &Vec::new(),
                    domains,
                ));
                TestKit {
                    world_state_view,
                    root_account_id: account_id,
                }
            }
        }

        #[test]
        fn test_initialize_dex_should_pass() {
            let mut testkit = TestKit::new();
            let dex_owner_public_key = KeyPair::generate()
                .expect("Failed to generate KeyPair.")
                .public_key;
            let dex_owner_account =
                Account::with_signatory("dex_owner", "Company", dex_owner_public_key);
            let dex_definition = DEXDefinition {
                id: DEXDefinitionId::new(DEX_NAME),
                owner_account_id: dex_owner_account.id.clone(),
            };
            let world_state_view = &mut testkit.world_state_view;
            let domain = world_state_view.peer().domains.get_mut("Company").unwrap();
            let register_account = domain.register_account(dex_owner_account.clone());
            register_account
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("failed to register dex owner account");
            let initialize_dex =
                initialize_dex(world_state_view.read_peer().id.clone(), &dex_definition);
            initialize_dex
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("failed to initialize dex");
            let dex_query = query_dex(DEXDefinitionId::new(&dex_definition.id.name));
            let query_result = dex_query
                .execute(&world_state_view)
                .expect("failed to query a dex");
            let decoded_dex_definition =
                decode_dex_definition(&query_result).expect("failed to decode a dex definition");
            assert_eq!(decoded_dex_definition, dex_definition);

            let dex_query = query_dex_collection(dex_owner_account.id.clone());
            let query_result = dex_query
                .execute(&world_state_view)
                .expect("failed to query dex collection");
            let decoded_dex_definitions: Vec<DEXDefinition> = decode_dex_collection(&query_result)
                .expect("failed to decode a dex collection")
                .collect();
            assert_eq!(&decoded_dex_definitions, &[dex_definition]);
        }

        // TODO: Test for multiple dex initialization

        #[test]
        fn test_register_dex_should_fail_with_account_not_found() {
            let mut testkit = TestKit::new();
            let dex_owner_public_key = KeyPair::generate()
                .expect("Failed to generate KeyPair.")
                .public_key;
            let dex_owner_account =
                Account::with_signatory("dex_owner", "Company", dex_owner_public_key);
            let dex_definition = DEXDefinition {
                id: DEXDefinitionId::new(DEX_NAME),
                owner_account_id: dex_owner_account.id,
            };
            let world_state_view = &mut testkit.world_state_view;
            let initialize_dex =
                initialize_dex(world_state_view.read_peer().id.clone(), &dex_definition);
            assert_eq!(
                initialize_dex
                    .execute(testkit.root_account_id.clone(), world_state_view)
                    .unwrap_err(),
                "Account not found."
            );
        }
    }
}

/// Query module provides functions for constructing DEX-related queries
/// and decoding the query results.
pub mod query {
    use super::*;

    /// Constructor of Iroha Query for retrieving collection of initialized DEX.
    pub fn query_dex_collection(dex_owner_id: <Account as Identifiable>::Id) -> IrohaQuery {
        crate::asset::query::GetAccountAssets::build_request(dex_owner_id).query
    }

    /// A helper function for decoding a collection of DEX definitions from the query result.
    ///
    /// Each `DEXDefinition` is encoded and stored in the DEX collection asset
    /// (`dex_collection_asset_definition_id`) store indexed by the name of a DEX. The given query
    /// result may not contain the above values, so this function can fail, returning `None`.
    pub fn decode_dex_collection<'a>(
        query_result: &'a QueryResult,
    ) -> Option<impl Iterator<Item = DEXDefinition> + 'a> {
        let account_assets_result = match query_result {
            QueryResult::GetAccountAssets(v) => v,
            _ => return None,
        };
        account_assets_result
            .assets
            .iter()
            .find(|asset| asset.id.definition_id == dex_collection_asset_definition_id())
            .map(|asset| {
                asset
                    .store
                    .values()
                    .filter_map(|data| DEXDefinition::decode(&mut data.as_slice()).ok())
            })
    }

    /// Constructor of Iroha Query for retrieving information about DEX.
    pub fn query_dex(dex_id: <DEXDefinition as Identifiable>::Id) -> IrohaQuery {
        crate::account::query::GetAccount::build_request(AccountId::new(
            DEX_ACCOUNT_NAME,
            &dex_id.name,
        ))
        .query
    }

    /// A helper function for decoding DEX definition from the query result.
    ///
    /// The `DEXDefinition` is encoded and stored in the DEX asset
    /// (`dex_asset_definition_id`) store under the
    /// `DEX_ASSET_DEX_DEFINITION_PARAMETER_KEY` key. The given query result may not
    /// contain the above values, so this function can fail, returning `None`.
    pub fn decode_dex_definition(query_result: &QueryResult) -> Option<DEXDefinition> {
        let account_result = match query_result {
            QueryResult::GetAccount(v) => v,
            _ => return None,
        };
        account_result
            .account
            .assets
            .iter()
            .find(|(id, _)| id.definition_id == dex_asset_definition_id())
            .and_then(|(_, asset)| {
                asset
                    .store
                    .get(DEX_ASSET_DEX_DEFINITION_PARAMETER_KEY)
                    .and_then(|data| DEXDefinition::decode(&mut data.as_slice()).ok())
            })
    }
}
