//! This module contains persistence related Iroha logic.
//! `Kura` is the main entity which should be used to store new `Block`s on the blockchain.

use crate::{merkle::MerkleTree, prelude::*};
use async_std::{
    fs::{metadata, File},
    prelude::*,
    task,
};
use iroha_derive::*;
use serde::Deserialize;
use std::{
    convert::TryFrom,
    fs,
    path::{Path, PathBuf},
};

/// High level data storage representation.
/// Provides all necessary methods to read and write data, hides implementation details.
#[derive(Debug)]
pub struct Kura {
    mode: Mode,
    //TODO: decide whether to store blockchain in WSV or in Kura, right now it is duplicated.
    /// Blockchain.
    pub blocks: Vec<ValidBlock>,
    block_store: BlockStore,
    block_sender: CommittedBlockSender,
    merkle_tree: MerkleTree,
}

#[allow(dead_code)]
impl Kura {
    /// Default `Kura` constructor.
    /// Kura will not be ready to work with before `init` method invocation.
    pub fn new(mode: Mode, block_store_path: &Path, block_sender: CommittedBlockSender) -> Self {
        Kura {
            mode,
            block_store: BlockStore::new(block_store_path),
            block_sender,
            merkle_tree: MerkleTree::new(),
            blocks: Vec::new(),
        }
    }

    pub fn from_configuration(
        configuration: &config::KuraConfiguration,
        block_sender: CommittedBlockSender,
    ) -> Self {
        Kura::new(
            configuration.kura_init_mode,
            Path::new(&configuration.kura_block_store_path),
            block_sender,
        )
    }

    /// `Kura` constructor with a [Genesis
    /// Block](https://en.wikipedia.org/wiki/Blockchain#cite_note-hadc-21).
    /// Kura will not be ready to work with before `init` method invocation.
    pub fn with_genesis_block(
        mode: Mode,
        block_store_path: &Path,
        block_sender: CommittedBlockSender,
        genesis_block: ValidBlock,
    ) -> Self {
        Kura {
            mode,
            block_store: BlockStore::with_genesis_block(block_store_path, genesis_block),
            block_sender,
            merkle_tree: MerkleTree::new(),
            blocks: Vec::new(),
        }
    }

    /// After constructing `Kura` it should be initialized to be ready to work with it.
    pub async fn init(&mut self) -> Result<(), String> {
        let blocks = self.block_store.read_all().await;
        self.merkle_tree =
            MerkleTree::new().build(&blocks.iter().map(|block| block.hash()).collect::<Vec<_>>());
        self.blocks = blocks;
        Ok(())
    }

    /// Methods consumes new validated block and atomically stores and caches it.
    #[log]
    pub async fn store(&mut self, block: ValidBlock) -> Result<Hash, String> {
        let block_store_result = self.block_store.write(&block).await;
        match block_store_result {
            Ok(hash) => {
                //TODO: shouldn't we add block hash to merkle tree here?
                self.block_sender.send(block.clone().commit()).await;
                self.blocks.push(block);
                Ok(hash)
            }
            Err(error) => {
                let blocks = self.block_store.read_all().await;
                self.merkle_tree = MerkleTree::new()
                    .build(&blocks.iter().map(|block| block.hash()).collect::<Vec<_>>());
                Err(error)
            }
        }
    }

    pub fn latest_block_hash(&self) -> Hash {
        self.blocks
            .last()
            .map(|block| block.hash())
            .unwrap_or([0u8; 32])
    }

    pub fn height(&self) -> u64 {
        self.blocks
            .last()
            .map(|block| block.header.height)
            .unwrap_or(0)
    }

    pub fn blocks_after(&self, hash: Hash) -> Option<&[ValidBlock]> {
        let from_pos = self
            .blocks
            .iter()
            .position(|block| block.header.previous_block_hash == hash)?;
        if self.blocks.len() > from_pos {
            Some(&self.blocks[from_pos..])
        } else {
            None
        }
    }
}

/// Kura work mode.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Strict validation of all blocks.
    Strict,
    /// Fast initialization with basic checks.
    Fast,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Strict
    }
}

/// Representation of a consistent storage.
#[derive(Debug)]
struct BlockStore {
    path: PathBuf,
}

impl BlockStore {
    fn new(path: &Path) -> BlockStore {
        if fs::read_dir(path).is_err() {
            fs::create_dir_all(path).expect("Failed to create Block Store directory.");
        }
        BlockStore {
            path: path.to_path_buf(),
        }
    }

    fn with_genesis_block(path: &Path, genesis_block: ValidBlock) -> BlockStore {
        let block_store = BlockStore::new(path);
        task::block_on(async { block_store.write(&genesis_block).await })
            .expect("Failed to write a Genesis Block.");
        block_store
    }

    fn get_block_filename(block_height: u64) -> String {
        format!("{}", block_height)
    }

    fn get_block_path(&self, block_height: u64) -> PathBuf {
        self.path.join(BlockStore::get_block_filename(block_height))
    }

    async fn write(&self, block: &ValidBlock) -> Result<Hash, String> {
        //filename is its height
        let path = self.get_block_path(block.header.height);
        match File::create(path).await {
            Ok(mut file) => {
                let hash = block.hash();
                let serialized_block: Vec<u8> = block.into();
                if let Err(error) = file.write_all(&serialized_block).await {
                    return Err(format!("Failed to write to storage file {}.", error));
                }
                Ok(hash)
            }
            Err(error) => Result::Err(format!("Failed to open storage file {}.", error)),
        }
    }

    async fn read(&self, height: u64) -> Result<ValidBlock, String> {
        let path = self.get_block_path(height);
        let mut file = File::open(&path).await.map_err(|_| "No file found.")?;
        let metadata = metadata(&path)
            .await
            .map_err(|_| "Unable to read metadata.")?;
        let mut buffer = vec![0; metadata.len() as usize];
        file.read(&mut buffer)
            .await
            .map_err(|_| "Buffer overflow.")?;
        Ok(ValidBlock::try_from(buffer).expect("Failed to read block from store."))
    }

    /// Returns a sorted vector of blocks starting from 0 height to the top block.
    async fn read_all(&self) -> Vec<ValidBlock> {
        let mut height = 0;
        let mut blocks = Vec::new();
        while let Ok(block) = self.read(height).await {
            blocks.push(block);
            height += 1;
        }
        blocks
    }
}

/// This module contains all configuration related logic.
pub mod config {
    use super::Mode;
    use serde::Deserialize;
    use std::{env, path::Path};

    const KURA_INIT_MODE: &str = "KURA_INIT_MODE";
    const KURA_BLOCK_STORE_PATH: &str = "KURA_BLOCK_STORE_PATH";
    const DEFAULT_KURA_BLOCK_STORE_PATH: &str = "./blocks";

    #[derive(Clone, Deserialize, Debug)]
    #[serde(rename_all = "UPPERCASE")]
    pub struct KuraConfiguration {
        /// Possible modes: `strict`, `fast`.
        #[serde(default)]
        pub kura_init_mode: Mode,
        /// Path to the existing block store folder or path to create new folder.
        #[serde(default = "default_kura_block_store_path")]
        pub kura_block_store_path: String,
    }

    impl KuraConfiguration {
        /// Set `kura_block_store_path` configuration parameter - will overwrite the existing one.
        ///
        /// # Panic
        /// If path is not valid this method will panic.
        pub fn kura_block_store_path(&mut self, path: &Path) {
            self.kura_block_store_path = path
                .to_str()
                .expect("Failed to yield slice from path")
                .to_string();
        }

        pub fn load_environment(&mut self) -> Result<(), String> {
            if let Ok(kura_init_mode) = env::var(KURA_INIT_MODE) {
                self.kura_init_mode = serde_json::from_str(&kura_init_mode)
                    .map_err(|e| format!("Failed to parse Kura Init Mode: {}", e))?;
            }
            if let Ok(kura_block_store_path) = env::var(KURA_BLOCK_STORE_PATH) {
                self.kura_block_store_path = kura_block_store_path;
            }
            Ok(())
        }
    }

    fn default_kura_block_store_path() -> String {
        DEFAULT_KURA_BLOCK_STORE_PATH.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{crypto::KeyPair, peer::PeerId};
    use async_std::sync;
    use tempfile::TempDir;

    #[async_std::test]
    async fn strict_init_kura() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir.");
        let (tx, _rx) = sync::channel(100);
        assert!(Kura::new(Mode::Strict, temp_dir.path(), tx)
            .init()
            .await
            .is_ok());
    }

    #[async_std::test]
    async fn strict_init_kura_with_genesis_block() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir.");
        let (tx, _rx) = sync::channel(100);
        let keypair = KeyPair::generate().expect("Failed to generate KeyPair.");
        let genesis_block = PendingBlock::new(Vec::new(), &keypair)
            .expect("Failed to create a block.")
            .chain_first()
            .validate(&WorldStateView::new(Peer::new(
                PeerId {
                    address: "127.0.0.1:8080".to_string(),
                    public_key: keypair.public_key,
                },
                &Vec::new(),
            )))
            .sign(&keypair)
            .expect("Failed to sign blocks.");
        let mut kura = Kura::with_genesis_block(Mode::Strict, temp_dir.path(), tx, genesis_block);
        assert!(kura.init().await.is_ok());
        assert!(kura.blocks.len() == 1);
    }

    #[async_std::test]
    async fn write_block_to_block_store() {
        let dir = tempfile::tempdir().unwrap();
        let keypair = KeyPair::generate().expect("Failed to generate KeyPair.");
        let block = PendingBlock::new(Vec::new(), &keypair)
            .expect("Failed to create a block.")
            .chain_first()
            .validate(&WorldStateView::new(Peer::new(
                PeerId {
                    address: "127.0.0.1:8080".to_string(),
                    public_key: keypair.public_key,
                },
                &Vec::new(),
            )))
            .sign(&keypair)
            .expect("Failed to sign blocks.");
        assert!(BlockStore::new(dir.path()).write(&block).await.is_ok());
    }

    #[async_std::test]
    async fn read_block_from_block_store() {
        let dir = tempfile::tempdir().unwrap();
        let keypair = KeyPair::generate().expect("Failed to generate KeyPair.");
        let block = PendingBlock::new(Vec::new(), &keypair)
            .expect("Failed to create a block.")
            .chain_first()
            .validate(&WorldStateView::new(Peer::new(
                PeerId {
                    address: "127.0.0.1:8080".to_string(),
                    public_key: keypair.public_key,
                },
                &Vec::new(),
            )))
            .sign(&keypair)
            .expect("Failed to sign blocks.");
        let block_store = BlockStore::new(dir.path());
        block_store
            .write(&block)
            .await
            .expect("Failed to write block to file.");
        assert!(block_store.read(0).await.is_ok())
    }

    #[async_std::test]
    async fn read_all_blocks_from_block_store() {
        let dir = tempfile::tempdir().unwrap();
        let block_store = BlockStore::new(dir.path());
        let n = 10;
        let keypair = KeyPair::generate().expect("Failed to generate KeyPair.");
        let mut block = PendingBlock::new(Vec::new(), &keypair)
            .expect("Failed to create a block.")
            .chain_first()
            .validate(&WorldStateView::new(Peer::new(
                PeerId {
                    address: "127.0.0.1:8080".to_string(),
                    public_key: keypair.public_key,
                },
                &Vec::new(),
            )))
            .sign(&keypair)
            .expect("Failed to sign blocks.");
        for height in 0..n {
            let hash = block_store
                .write(&block)
                .await
                .expect("Failed to write block to file.");
            block = PendingBlock::new(Vec::new(), &keypair)
                .expect("Failed to create a block.")
                .chain(height, hash, 0, Vec::new())
                .validate(&WorldStateView::new(Peer::new(
                    PeerId {
                        address: "127.0.0.1:8080".to_string(),
                        public_key: keypair.public_key,
                    },
                    &Vec::new(),
                )))
                .sign(&keypair)
                .expect("Failed to sign blocks.");
        }
        let blocks = block_store.read_all().await;
        assert_eq!(blocks.len(), n as usize)
    }

    ///Kura takes as input blocks, which comprise multiple transactions. Kura is meant to take only
    ///blocks as input that have passed stateless and stateful validation, and have been finalized
    ///by consensus. For finalized blocks, Kura simply commits the block to the block storage on
    ///the block_store and updates atomically the in-memory hashmaps that make up the key-value store that
    ///is the world-state-view. To optimize networking syncing, which works on 100 block chunks,
    ///chunks of 100 blocks each are stored in files in the block store.
    #[async_std::test]
    async fn store_block() {
        let keypair = KeyPair::generate().expect("Failed to generate KeyPair.");
        let block = PendingBlock::new(Vec::new(), &keypair)
            .expect("Failed to create a block.")
            .chain_first()
            .validate(&WorldStateView::new(Peer::new(
                PeerId {
                    address: "127.0.0.1:8080".to_string(),
                    public_key: keypair.public_key,
                },
                &Vec::new(),
            )))
            .sign(&keypair)
            .expect("Failed to sign blocks.");
        let dir = tempfile::tempdir().unwrap();
        let (tx, _rx) = sync::channel(100);
        let mut kura = Kura::new(Mode::Strict, dir.path(), tx);
        kura.init().await.expect("Failed to init Kura.");
        kura.store(block)
            .await
            .expect("Failed to store block into Kura.");
    }
}
