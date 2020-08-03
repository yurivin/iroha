//! Helper module to build a genesis configuration for the Offchain Worker

use super::{
    AccountId, BalancesConfig, GenesisConfig, Signature, SudoConfig, SystemConfig, XORConfig,
    WASM_BINARY,
};
use sp_core::{crypto::AccountId32, ecdsa, ed25519, sr25519, Pair};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Helper function to generate a crypto pair from seed
fn get_from_seed<TPair: Pair>(seed: &str) -> TPair::Public {
    let pair =
        TPair::from_string(&format!("//{}", seed), None).expect("static values are valid; qed");
    frame_support::debug::info!("{}: {:?}", seed, pair.public().as_ref());
    pair.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn account_id_from_seed<TPair: Pair>(seed: &str) -> AccountId
where
    AccountPublic: From<TPair::Public>,
{
    use parity_scale_codec::Encode;
    let acc = AccountPublic::from(get_from_seed::<TPair>(seed)).into_account();
    let vec = acc.encode();
    println!("ACC {}: {:?}", seed, vec);
    acc
}

pub fn dev_genesis() -> GenesisConfig {
    testnet_genesis(
        // Root Key
        account_id_from_seed::<sr25519::Pair>("Alice"),
        // account_id_from_seed::<sr25519::Pair>("Alice"),
        // Endowed Accounts
        vec![
            account_id_from_seed::<sr25519::Pair>("Alice"),
            // account_id_from_seed::<sr25519::Pair>("Alice"),
            AccountId32::from([
                52u8, 45, 84, 67, 137, 84, 47, 252, 35, 59, 237, 44, 144, 70, 71, 206, 243, 67, 8,
                115, 247, 189, 204, 26, 181, 226, 232, 81, 123, 12, 81, 120,
            ]),
            // AccountId32::from([0x88u8, 0xdc, 0x34, 0x17, 0xd5, 0x05, 0x8e, 0xc4, 0xb4, 0x50, 0x3e, 0x0c, 0x12, 0xea, 0x1a, 0x0a, 0x89, 0xbe, 0x20, 0x0f, 0xe9, 0x89, 0x22, 0x42, 0x3d, 0x43, 0x34, 0x01, 0x4f, 0xa6, 0xb0, 0xee]),
            account_id_from_seed::<sr25519::Pair>("Bob"),
            account_id_from_seed::<sr25519::Pair>("Alice//stash"),
            account_id_from_seed::<sr25519::Pair>("root//stash"),
        ],
    )
}
//0x00000000000000001000000000000000
//0x00000000000000001000000000000000
/// Helper function to build a genesis configuration
pub fn testnet_genesis(root_key: AccountId, endowed_accounts: Vec<AccountId>) -> GenesisConfig {
    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances_Instance1: Some(XORConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .inspect(|x| {
                    dbg!(x);
                })
                .filter(|x| {
                    x != &AccountId32::from([
                        52u8, 45, 84, 67, 137, 84, 47, 252, 35, 59, 237, 44, 144, 70, 71, 206, 243,
                        67, 8, 115, 247, 189, 204, 26, 181, 226, 232, 81, 123, 12, 81, 120,
                    ])
                })
                .inspect(|x| {
                    dbg!(x);
                })
                .map(|k| (k, 1 << 60))
                .collect(),
        }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        }),
        pallet_sudo: Some(SudoConfig { key: root_key }),
    }
}
