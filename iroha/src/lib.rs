//! Iroha - A simple, enterprise-grade decentralized ledger.

#![warn(missing_docs)]
#![warn(private_doc_tests)]
pub mod account;
pub mod asset;
pub mod block;
pub mod block_sync;
#[cfg(feature = "bridge")]
pub mod bridge;
pub mod config;
pub mod crypto;
#[cfg(feature = "dex")]
pub mod dex;
pub mod domain;
pub mod event;
pub mod isi;
mod kura;
pub mod maintenance;
mod merkle;
pub mod peer;
pub mod permission;
pub mod query;
mod queue;
pub mod sumeragi;
pub mod torii;
pub mod tx;
pub mod wsv;

use crate::bridge::asset::ExternalAsset;
use crate::domain::isi::DomainInstruction;
use crate::peer::isi::PeerInstruction;
use crate::tx::Payload;
use crate::{
    block_sync::{message::Message as BlockSyncMessage, BlockSynchronizer},
    config::Configuration,
    kura::Kura,
    maintenance::System,
    peer::{Peer, PeerId},
    permission::Permission,
    prelude::*,
    queue::Queue,
    sumeragi::{message::Message as SumeragiMessage, Sumeragi},
    torii::Torii,
};
use async_std::{
    prelude::*,
    sync::{self, Receiver, RwLock, Sender},
    task,
};
use std::time::SystemTime;
use std::{collections::BTreeMap, sync::Arc, time::Duration};

/// The interval at which sumeragi checks if there are tx in the `queue`.
pub const TX_RETRIEVAL_INTERVAL: Duration = Duration::from_millis(100);

/// Type of `Sender<ValidBlock>` which should be used for channels of `ValidBlock` messages.
pub type ValidBlockSender = Sender<ValidBlock>;
/// Type of `Receiver<ValidBlock>` which should be used for channels of `ValidBlock` messages.
pub type ValidBlockReceiver = Receiver<ValidBlock>;
/// Type of `Sender<CommittedBlock>` which should be used for channels of `CommittedBlock` messages.
pub type CommittedBlockSender = Sender<CommittedBlock>;
/// Type of `Receiver<CommittedBlock>` which should be used for channels of `CommittedBlock` messages.
pub type CommittedBlockReceiver = Receiver<CommittedBlock>;
/// Type of `Sender<AcceptedTransaction>` which should be used for channels of `AcceptedTransaction` messages.
pub type TransactionSender = Sender<AcceptedTransaction>;
/// Type of `Receiver<AcceptedTransaction>` which should be used for channels of
/// `AcceptedTransaction` messages.
pub type TransactionReceiver = Receiver<AcceptedTransaction>;
/// Type of `Sender<Message>` which should be used for channels of `Message` messages.
pub type SumeragiMessageSender = Sender<SumeragiMessage>;
/// Type of `Receiver<Message>` which should be used for channels of `Message` messages.
pub type SumeragiMessageReceiver = Receiver<SumeragiMessage>;
/// Type of `Sender<Message>` which should be used for channels of `Message` messages.
pub type BlockSyncMessageSender = Sender<BlockSyncMessage>;
/// Type of `Receiver<Message>` which should be used for channels of `Message` messages.
pub type BlockSyncMessageReceiver = Receiver<BlockSyncMessage>;

/// Iroha is an [Orchestrator](https://en.wikipedia.org/wiki/Orchestration_%28computing%29) of the
/// system. It configure, coordinate and manage transactions and queries processing, work of consensus and storage.
pub struct Iroha {
    torii: Arc<RwLock<Torii>>,
    queue: Arc<RwLock<Queue>>,
    sumeragi: Arc<RwLock<Sumeragi>>,
    kura: Arc<RwLock<Kura>>,
    transactions_receiver: Arc<RwLock<TransactionReceiver>>,
    wsv_blocks_receiver: Arc<RwLock<CommittedBlockReceiver>>,
    kura_blocks_receiver: Arc<RwLock<ValidBlockReceiver>>,
    sumeragi_message_receiver: Arc<RwLock<SumeragiMessageReceiver>>,
    block_sync_message_receiver: Arc<RwLock<BlockSyncMessageReceiver>>,
    world_state_view: Arc<RwLock<WorldStateView>>,
    block_sync: Arc<RwLock<BlockSynchronizer>>,
}

impl Iroha {
    /// Default `Iroha` constructor used to build it based on the provided `Configuration`.
    pub fn new(config: Configuration) -> Self {
        iroha_logger::init(&config.logger_configuration).expect("Failed to initialize logger.");
        log::info!("Configuration: {:?}", config);
        let (transactions_sender, transactions_receiver) = sync::channel(100);
        let (wsv_blocks_sender, wsv_blocks_receiver) = sync::channel(100);
        let (kura_blocks_sender, kura_blocks_receiver) = sync::channel(100);
        let (sumeragi_message_sender, sumeragi_message_receiver) = sync::channel(100);
        let (block_sync_message_sender, block_sync_message_receiver) = sync::channel(100);
        let (events_sender, events_receiver) = sync::channel(100);
        let domain_name = "global".to_string();
        let mut asset_definitions = BTreeMap::new();
        let permission_asset_definition_id = permission::permission_asset_definition_id();
        asset_definitions.insert(
            permission_asset_definition_id.clone(),
            AssetDefinition::new(permission_asset_definition_id.clone()),
        );
        let account_id = AccountId::new("root", &domain_name);
        let asset_id = AssetId {
            definition_id: permission_asset_definition_id.clone(),
            account_id: account_id.clone(),
        };
        let asset = Asset::with_permission(asset_id.clone(), Permission::Anything);
        println!("PK: {:?}", config.public_key.inner);
        println!("SK: {:?}", config.private_key.inner);
        let mut account =
            Account::with_signatory(&account_id.name, &account_id.domain_name, config.public_key);
        account.assets.insert(asset_id, asset);
        let mut accounts = BTreeMap::new();
        accounts.insert(account_id.clone(), account);

        let bridge_admin_account_id = AccountId::new("bridge_admin", &domain_name);
        let bridge_admin_asset_id = AssetId {
            definition_id: permission_asset_definition_id,
            account_id: bridge_admin_account_id.clone(),
        };
        let bridge_permissions_asset =
            Asset::with_permission(bridge_admin_asset_id.clone(), Permission::Anything);
        let bpk = PublicKey {
            inner: [
                52, 80, 113, 218, 85, 229, 220, 206, 250, 170, 68, 3, 57, 65, 94, 249, 242, 102,
                51, 56, 163, 143, 125, 160, 223, 33, 190, 90, 180, 224, 85, 239,
            ],
        };
        let bsk = PrivateKey {
            inner: vec![
                250, 199, 149, 157, 191, 231, 47, 5, 46, 90, 12, 60, 141, 101, 48, 242, 2, 176, 47,
                216, 249, 245, 202, 53, 128, 236, 141, 235, 119, 151, 71, 158, 52, 80, 113, 218,
                85, 229, 220, 206, 250, 170, 68, 3, 57, 65, 94, 249, 242, 102, 51, 56, 163, 143,
                125, 160, 223, 33, 190, 90, 180, 224, 85, 239,
            ],
        };
        // println!("pk {:?}", bpk.inner);
        // println!("sk {:?}", bsk.inner);

        let mut bridge_admin_account = Account::with_signatory(
            &bridge_admin_account_id.name,
            &bridge_admin_account_id.domain_name,
            bpk,
        );
        bridge_admin_account
            .assets
            .insert(bridge_admin_asset_id, bridge_permissions_asset);
        accounts.insert(bridge_admin_account_id.clone(), bridge_admin_account);
        let domain = Domain {
            name: domain_name.clone(),
            accounts,
            asset_definitions,
        };
        let mut domains = BTreeMap::new();
        {
            let bridge_domain_name = "bridge".to_string();
            let mut bridge_asset_definitions = BTreeMap::new();
            let asset_definition_ids = [
                bridge::bridges_asset_definition_id(),
                bridge::bridge_asset_definition_id(),
                bridge::bridge_external_assets_asset_definition_id(),
                bridge::bridge_incoming_external_transactions_asset_definition_id(),
                bridge::bridge_outgoing_external_transactions_asset_definition_id(),
            ];
            for asset_definition_id in &asset_definition_ids {
                bridge_asset_definitions.insert(
                    asset_definition_id.clone(),
                    AssetDefinition::new(asset_definition_id.clone()),
                );
            }
            let bridge_domain = Domain {
                name: bridge_domain_name.clone(),
                accounts: BTreeMap::new(),
                asset_definitions: bridge_asset_definitions,
            };
            // domains.insert(domain_name, domain);
            domains.insert(bridge_domain_name, bridge_domain);
        }
        let dmn = domain.clone();
        domains.insert(domain_name, domain);
        let world_state_view = Arc::new(RwLock::new(WorldStateView::new(Peer::with_domains(
            PeerId::new(&config.torii_configuration.torii_url, &config.public_key),
            &config.sumeragi_configuration.trusted_peers,
            domains,
        ))));
        let torii = Torii::from_configuration(
            &config.torii_configuration,
            Arc::clone(&world_state_view),
            transactions_sender.clone(),
            sumeragi_message_sender,
            block_sync_message_sender,
            System::new(&config),
            (events_sender.clone(), events_receiver),
        );
        let tx_sender = transactions_sender.clone();
        let cfg = config.clone();
        task::spawn(async move {
            task::sleep(Duration::from_secs(5)).await;
            let (pk, sk) = (bpk, bsk);
            // let (pk, sk) = cfg.key_pair();
            let peer_id = PeerId::new(&cfg.torii_configuration.torii_url, &cfg.public_key);
            // let isi = Instruction::Peer(PeerInstruction::AddDomain(
            //     "saf".into(),
            //     PeerId::new(&cfg.torii_configuration.torii_url, &cfg.public_key),
            // ));
            let bridge_domain_name = "polkadot".to_string();
            let bridge_def_id = BridgeDefinitionId {
                name: bridge_domain_name.clone(),
            };
            let bridge_def = BridgeDefinition {
                id: bridge_def_id.clone(),
                kind: BridgeKind::IClaim,
                owner_account_id: bridge_admin_account_id.clone(),
            };
            let ext_asset = ExternalAsset {
                bridge_id: BridgeId::new(&bridge_def_id.name),
                name: "DOT".to_string(),
                id: "DOT".to_string(),
                decimals: 10,
            };
            let register_bridge = bridge::isi::register_bridge(peer_id, &bridge_def);
            let register_client = bridge::isi::add_client(&bridge_def_id, pk.clone());
            let dot_asset_def = AssetDefinition::new(AssetDefinitionId {
                name: "DOT".to_string(),
                domain_name: bridge_domain_name.clone(),
            });
            let register_dot_asset =
                Register::new(dot_asset_def, bridge_domain_name.clone()).into();
            let xor_asset_def = AssetDefinition::new(AssetDefinitionId {
                name: "XOR".to_string(),
                domain_name: "global".into(),
            });
            let register_xor_asset = Register::new(xor_asset_def.clone(), dmn.name.clone()).into();
            let register_ext_asset = bridge::isi::register_external_asset(&ext_asset);
            let mint_xor = Mint::new(
                100u32,
                AssetId::new(xor_asset_def.id.clone(), account_id.clone()),
            )
            .into();
            let bridge_account_id = AccountId::new("bridge", "polkadot");
            let transfer_xor = Transfer::new(
                account_id.clone(),
                Asset::with_quantity(
                    AssetId::new(xor_asset_def.id.clone(), account_id.clone()),
                    100,
                ),
                bridge_account_id.clone(),
            )
            .into();
            let kp = KeyPair {
                public_key: pk,
                private_key: sk,
            };
            let payload = Payload {
                instructions: vec![
                    register_xor_asset,
                    register_bridge,
                    register_client,
                    register_dot_asset,
                    register_ext_asset,
                    mint_xor,
                    transfer_xor,
                ],
                account_id: bridge_admin_account_id,
                creation_time: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Failed to get System Time.")
                    .as_millis() as u64,
                time_to_live_ms: 10000,
            };
            let sig = Signature::new(kp, &Vec::from(&payload)).unwrap();
            let tx = AcceptedTransaction {
                payload,
                signatures: vec![sig],
            };
            tx_sender.send(tx).await;
        });
        let kura = Kura::from_configuration(&config.kura_configuration, wsv_blocks_sender);
        let sumeragi = Arc::new(RwLock::new(
            Sumeragi::from_configuration(
                &config.sumeragi_configuration,
                Arc::new(RwLock::new(kura_blocks_sender)),
                events_sender,
                world_state_view.clone(),
                transactions_sender.clone(),
                kura.latest_block_hash(),
                kura.height(),
            )
            .expect("Failed to initialize Sumeragi."),
        ));
        let kura = Arc::new(RwLock::new(kura));
        let block_sync = Arc::new(RwLock::new(BlockSynchronizer::from_configuration(
            &config.block_sync_configuration,
            kura.clone(),
            sumeragi.clone(),
            PeerId::new(&config.torii_configuration.torii_url, &config.public_key),
        )));
        let queue = Arc::new(RwLock::new(Queue::from_configuration(
            &config.queue_configuration,
        )));
        Iroha {
            queue,
            torii: Arc::new(RwLock::new(torii)),
            sumeragi,
            kura,
            world_state_view,
            transactions_receiver: Arc::new(RwLock::new(transactions_receiver)),
            wsv_blocks_receiver: Arc::new(RwLock::new(wsv_blocks_receiver)),
            sumeragi_message_receiver: Arc::new(RwLock::new(sumeragi_message_receiver)),
            kura_blocks_receiver: Arc::new(RwLock::new(kura_blocks_receiver)),
            block_sync_message_receiver: Arc::new(RwLock::new(block_sync_message_receiver)),
            block_sync,
        }
    }

    /// To make `Iroha` peer work it should be started first. After that moment it will listen for
    /// incoming requests and messages.
    #[allow(clippy::eval_order_dependence)]
    pub async fn start(&self) -> Result<(), String> {
        //TODO: ensure the initialization order of `Kura` and `WSV`.
        let kura = Arc::clone(&self.kura);
        kura.write().await.init().await?;
        let world_state_view = Arc::clone(&self.world_state_view);
        world_state_view
            .write()
            .await
            .init(&kura.read().await.blocks);
        let torii = Arc::clone(&self.torii);
        let torii_handle = task::spawn(async move {
            if let Err(e) = torii.write().await.start().await {
                log::error!("Failed to start Torii: {}", e);
            }
        });
        self.block_sync.read().await.start();
        let transactions_receiver = Arc::clone(&self.transactions_receiver);
        let queue = Arc::clone(&self.queue);
        let tx_handle = task::spawn(async move {
            println!("enter");
            while let Some(transaction) = transactions_receiver.write().await.next().await {
                println!("tx handle");
                queue.write().await.push_pending_transaction(transaction);
            }
        });
        let queue = Arc::clone(&self.queue);
        let sumeragi = Arc::clone(&self.sumeragi);
        let voting_handle = task::spawn(async move {
            loop {
                if !sumeragi.write().await.voting_in_progress().await {
                    sumeragi
                        .write()
                        .await
                        .round(queue.write().await.pop_pending_transactions())
                        .await
                        .expect("Round failed.");
                }
                task::sleep(TX_RETRIEVAL_INTERVAL).await;
            }
        });
        let wsv_blocks_receiver = Arc::clone(&self.wsv_blocks_receiver);
        let world_state_view = Arc::clone(&self.world_state_view);
        let sumeragi = Arc::clone(&self.sumeragi);
        let block_sync = Arc::clone(&self.block_sync);
        let wsv_handle = task::spawn(async move {
            while let Some(block) = wsv_blocks_receiver.write().await.next().await {
                world_state_view.write().await.put(&block);
                sumeragi.write().await.update_network_topology().await;
                block_sync.write().await.continue_sync().await;
            }
        });
        let sumeragi_message_receiver = Arc::clone(&self.sumeragi_message_receiver);
        let sumeragi = Arc::clone(&self.sumeragi);
        let sumeragi_message_handle = task::spawn(async move {
            while let Some(message) = sumeragi_message_receiver.write().await.next().await {
                if let Err(e) = message.handle(&mut *sumeragi.write().await).await {
                    log::error!("Handle message failed: {}", e);
                }
            }
        });
        let block_sync_message_receiver = Arc::clone(&self.block_sync_message_receiver);
        let block_sync = Arc::clone(&self.block_sync);
        let block_sync_message_handle = task::spawn(async move {
            while let Some(message) = block_sync_message_receiver.write().await.next().await {
                message.handle(&mut *block_sync.write().await).await;
            }
        });
        let kura_blocks_receiver = Arc::clone(&self.kura_blocks_receiver);
        let kura = Arc::clone(&self.kura);
        let kura_handle = task::spawn(async move {
            while let Some(block) = kura_blocks_receiver.write().await.next().await {
                let _hash = kura
                    .write()
                    .await
                    .store(block)
                    .await
                    .expect("Failed to write block.");
            }
        });
        futures::join!(
            torii_handle,
            kura_handle,
            voting_handle,
            wsv_handle,
            sumeragi_message_handle,
            tx_handle,
            block_sync_message_handle,
        );
        Ok(())
    }
}

/// This trait marks entity that implement it as identifiable with an `Id` type to find them by.
pub trait Identifiable {
    /// Defines the type of entity's identification.
    type Id;
}

pub mod prelude {
    //! Re-exports important traits and types. Meant to be glob imported when using `Iroha`.

    #[doc(inline)]
    pub use crate::{
        account::{Account, Id as AccountId},
        asset::{Asset, AssetDefinition, AssetDefinitionId, AssetId},
        block::{CommittedBlock, PendingBlock, ValidBlock},
        crypto::{Hash, KeyPair, PrivateKey, PublicKey, Signature},
        domain::Domain,
        isi::{Add, Demint, Instruction, Mint, Register, Remove, Transfer},
        peer::{Peer, PeerId},
        query::{IrohaQuery, Query, QueryRequest, QueryResult},
        tx::{AcceptedTransaction, RequestedTransaction, SignedTransaction, ValidTransaction},
        wsv::WorldStateView,
        CommittedBlockReceiver, CommittedBlockSender, Identifiable, Iroha, TransactionReceiver,
        TransactionSender, ValidBlockReceiver, ValidBlockSender,
    };

    #[doc(inline)]
    #[cfg(feature = "bridge")]
    pub use crate::bridge::{Bridge, BridgeDefinition, BridgeDefinitionId, BridgeId, BridgeKind};
}
