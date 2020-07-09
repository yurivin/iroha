//! This module contains functionality related to `DEX`.

use crate::prelude::*;
use iroha_derive::Io;
use parity_scale_codec::{Decode, Encode};

const DEX_DOMAIN_NAME: &str = "dex";

type Name = String;

// TODO[modbrin]: add tests in permission.rs

/// Identification of a DEX definition.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Encode, Decode)]
pub struct DEXDefinitionId {
    /// Domain name to which given DEX belongs.
    pub domain_name: Name,
}

impl DEXDefinitionId {
    /// Default DEX definition identifier constructor.
    pub fn new(domain_name: &str) -> Self {
        DEXDefinitionId {
            domain_name: domain_name.to_owned(),
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

/// Identification of a DEX, consists of it's definition id.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Encode, Decode)]
pub struct DEXId {
    definition_id: <DEXDefinition as Identifiable>::Id,
}

impl DEXId {
    /// Default DEX identifier constructor.
    pub fn new(domain_name: &str) -> Self {
        DEXId {
            definition_id: DEXDefinitionId::new(domain_name),
        }
    }
}

/// `DEX` entity encapsulates data and logic for management of
/// decentralized exchanges in domains.
#[derive(Encode, Decode, Debug, Clone)]
pub struct DEX {
    /// An identification of the `DEX`.
    pub id: <DEX as Identifiable>::Id,
}

impl DEX {
    /// Default `DEX` entity constructor.
    pub fn new(domain_name: &str) -> Self {
        DEX {
            id: DEXId::new(domain_name),
        }
    }
}

impl Identifiable for DEX {
    type Id = DEXId;
}

/// Iroha Special Instructions module provides helper-methods for `Peer` for operating DEX,
/// Token Pairs and Liquidity Sources.
pub mod isi {
    use super::*;
    use crate::isi::prelude::*;
    use crate::permission::isi::PermissionInstruction;
    use crate::permission::permission_asset_definition_id;

    impl Peer {
        /// Constructor of `Register<Peer, DEXDefinition>` Iroha Special Instruction.
        pub fn initialize_dex(&self, object: DEXDefinition) -> Register<Peer, DEXDefinition> {
            Register {
                object,
                destination_id: self.id.clone(),
            }
        }
    }

    impl Register<Peer, DEXDefinition> {
        /// Registers the `DEX` by its definition on the given `WorldStateView`.
        ///
        /// Constructs `DEX` entity according to provided `DEXDefinition` and
        /// initializes it in specified domain.
        pub(crate) fn execute(self, world_state_view: &mut WorldStateView) -> Result<(), String> {
            let dex_definition = self.object;
            let dex_owner_account = world_state_view
                .read_account(&dex_definition.owner_account_id)
                .ok_or("Account not found.")?
                .clone();
            PermissionInstruction::CanManageDEX(dex_owner_account.id).execute(world_state_view)?;
            let domain_name = &dex_definition.id.domain_name;
            let dex = DEX::new(domain_name);
            world_state_view.initialize_dex(dex)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
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
                    ..Default::default()
                };
                let dex_domain = Domain {
                    name: DEX_DOMAIN_NAME.to_owned(),
                    accounts: BTreeMap::new(),
                    asset_definitions: BTreeMap::new(),
                    ..Default::default()
                };
                let mut domains = BTreeMap::new();
                domains.insert(domain_name.clone(), domain);
                domains.insert(DEX_DOMAIN_NAME.to_owned(), dex_domain);
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
            // create dex owner account
            let dex_owner_public_key = KeyPair::generate()
                .expect("Failed to generate KeyPair.")
                .public_key;
            let mut dex_owner_account =
                Account::with_signatory("dex_owner", "dex", dex_owner_public_key);

            // create permissions to operate any dex
            let asset_definition = permission_asset_definition_id();
            let asset_id = AssetId {
                definition_id: asset_definition,
                account_id: dex_owner_account.id.clone(),
            };
            let domain_name = Name::from("Company");
            let manage_dex_permission =
                Asset::with_permission(asset_id.clone(), Permission::ManageDEX);
            dex_owner_account
                .assets
                .insert(asset_id.clone(), manage_dex_permission);

            // create dex definition
            let dex_definition = DEXDefinition {
                id: DEXDefinitionId::new(&domain_name),
                owner_account_id: dex_owner_account.id.clone(),
            };
            let world_state_view = &mut testkit.world_state_view;
            let domain = world_state_view
                .peer()
                .domains
                .get_mut(DEX_DOMAIN_NAME)
                .unwrap();

            // register dex owner account
            let register_account = domain.register_account(dex_owner_account.clone());
            register_account
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("failed to register dex owner account");

            world_state_view
                .peer
                .initialize_dex(dex_definition.clone())
                .execute(world_state_view)
                .expect("failed to initialize dex");

            let query_result = world_state_view
                .read_dex(&domain_name)
                .expect("query dex failed");
            assert_eq!(&query_result.id.definition_id.domain_name, &domain_name);
        }

        // TODO[modbrin]: add negative tests
    }
}

/// Iroha World State View module provides extensions for the `WorldStateView` for initializing
/// a DEX in particular Domain.
pub mod wsv {
    use super::*;

    impl WorldStateView {
        /// Initialize the `DEX` for domain.
        pub fn initialize_dex(&mut self, dex: DEX) -> Result<(), String> {
            let domain_name = &dex.id.definition_id.domain_name;
            let domain = self
                .peer()
                .domains
                .get_mut(domain_name)
                .ok_or("domain not found")?;
            if domain.dex.is_none() {
                domain.dex = Some(dex);
                Ok(())
            } else {
                Err("dex is already initialized for domain".to_owned())
            }
        }

        /// Get `DEX` without an ability to modify it.
        pub fn read_dex(&self, domain_name: &str) -> Option<&DEX> {
            self.read_peer().domains.get(domain_name)?.dex.as_ref()
        }

        /// Get `DEX` with an ability to modify it.
        pub fn dex(&mut self, domain_name: &str) -> Option<&mut DEX> {
            self.peer().domains.get_mut(domain_name)?.dex.as_mut()
        }
    }
}
