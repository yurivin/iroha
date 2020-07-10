//! This module contains functionality related to `DEX`.

use crate::prelude::*;
use iroha_derive::Io;
use parity_scale_codec::{Decode, Encode};

const DEX_DOMAIN_NAME: &str = "dex";

type Name = String;

/// Identification of a DEX, consists of its domain name.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Encode, Decode)]
pub struct DEXId {
    domain_name: Name,
}

impl DEXId {
    /// Default DEX identifier constructor.
    pub fn new(domain_name: &str) -> Self {
        DEXId {
            domain_name: domain_name.to_owned(),
        }
    }
}

/// `DEX` entity encapsulates data and logic for management of
/// decentralized exchanges in domains.
#[derive(Encode, Decode, Debug, Clone)]
pub struct DEX {
    /// An identification of the `DEX`.
    pub id: <DEX as Identifiable>::Id,
    /// DEX owner's account Identification. Only this account will be able to manipulate the DEX.
    pub owner_account_id: <Account as Identifiable>::Id,
}

impl DEX {
    /// Default `DEX` entity constructor.
    pub fn new(domain_name: &str, owner_account_id: <Account as Identifiable>::Id) -> Self {
        DEX {
            id: DEXId::new(domain_name),
            owner_account_id,
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

    impl Peer {
        /// Constructor of `Register<Peer, DEX>` Iroha Special Instruction.
        pub fn initialize_dex(
            &self,
            domain_name: &str,
            owner_account_id: <Account as Identifiable>::Id,
        ) -> Register<Peer, DEX> {
            Register {
                object: DEX::new(domain_name, owner_account_id),
                destination_id: self.id.clone(),
            }
        }
    }

    impl Register<Peer, DEX> {
        /// Registers the `DEX` entity with specified parameters on the given `WorldStateView`.
        pub(crate) fn execute(
            &self,
            authority: <Account as Identifiable>::Id,
            world_state_view: &mut WorldStateView,
        ) -> Result<(), String> {
            PermissionInstruction::CanManageDEX(authority).execute(world_state_view)?;
            let dex = self.object.clone();
            world_state_view
                .read_account(&dex.owner_account_id)
                .ok_or("Account not found.")?;
            world_state_view.initialize_dex(dex)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::peer::PeerId;
        use crate::permission::{permission_asset_definition_id, Permission};
        use std::collections::BTreeMap;

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

            // get world state view and dex domain
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
                .initialize_dex(&domain_name, dex_owner_account.id.clone())
                .execute(dex_owner_account.id.clone(), world_state_view)
                .expect("failed to initialize dex");

            let query_result = world_state_view
                .read_dex(&domain_name)
                .expect("query dex failed");
            assert_eq!(&query_result.id.domain_name, &domain_name);
        }

        #[test]
        fn test_initialize_dex_should_fail_with_permission_not_found() {
            let mut testkit = TestKit::new();
            // create dex owner account
            let dex_owner_public_key = KeyPair::generate()
                .expect("Failed to generate KeyPair.")
                .public_key;
            let mut dex_owner_account =
                Account::with_signatory("dex_owner", "dex", dex_owner_public_key);
            let domain_name = Name::from("Company");

            // get world state view and dex domain
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

            assert_eq!(
                world_state_view
                    .peer
                    .initialize_dex(&domain_name, dex_owner_account.id.clone())
                    .execute(dex_owner_account.id.clone(), world_state_view)
                    .unwrap_err(),
                format!("Error: Permission not found., CanManageDEX(Id {{ name: \"{}\", domain_name: \"{}\" }})",
                    "dex_owner", DEX_DOMAIN_NAME)
            );
        }
    }
}

/// Iroha World State View module provides extensions for the `WorldStateView` for initializing
/// a DEX in particular Domain.
pub mod wsv {
    use super::*;

    impl WorldStateView {
        /// Initialize the `DEX` for domain.
        pub fn initialize_dex(&mut self, dex: DEX) -> Result<(), String> {
            let domain_name = &dex.id.domain_name;
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
