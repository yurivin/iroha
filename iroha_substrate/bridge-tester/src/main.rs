use codec::{Decode, Encode};
use keyring::ed25519::{ed25519::Pair as EdPair, Keyring};
use keyring::sr25519::sr25519::Pair as SrPair;

use iroha::prelude::*;
use iroha_client::{
    client::{account::by_id, Client},
    config::Configuration,
};
use sp_core::{crypto::Pair};
use sp_runtime::AccountId32;

use iroha::asset::query::GetAccountAssetsResult;
use iroha::permission::Permission::Anything;
use iroha::permission::Permissions;
use substrate_api_client::{compose_extrinsic, extrinsic::xt_primitives::UncheckedExtrinsicV4,
                           utils::hexstr_to_vec, Api, XtStatus, AccountData, AccountInfo};
use sp_core::crypto::Ss58Codec;

fn check_response_assets(
    response: &QueryResult,
    expected_xor_amount: u32,
) {
    if let QueryResult::GetAccount(get_account_result) = response {
        let account = &get_account_result.account;
        let assets = &account.assets;
        let xor_amount = assets
            .iter()
            .find(|(_, asset)| asset.id.definition_id.name == "XOR")
            .map(|(_, asset)| asset.quantity)
            .unwrap_or(0);
        assert_eq!(xor_amount, expected_xor_amount);
        println!(
            "{} account balance on Iroha is: {} XOR",
            account.id, expected_xor_amount
        );
    } else {
        panic!("Test failed.");
    }
}

#[async_std::main]
async fn main() {
    let configuration =
        Configuration::from_path("config.json").expect("Failed to load configuration.");
    let mut iroha_client = Client::new(&configuration);
    let url = "127.0.0.1:9944";
    let seed = "Alice";
    let signer = SrPair::from_string(&format!("//{}", seed), None).unwrap();
    let api = Api::new(format!("ws://{}", url)).set_signer(signer);
    let xt: UncheckedExtrinsicV4<_> =
        compose_extrinsic!(api.clone(), "TemplateModule", "fetch_blocks_signed");

    println!("[BRIDGE TEST] Checking account balances before the transfer...");
    // check assets before the transfer, but after the user sent transaction to the bridge
    let bridge_account_id = AccountId::new("bridge", "polkadot");
    let get_bridge_account = by_id(bridge_account_id);
    let response = iroha_client
        .request(&get_bridge_account)
        .await
        .expect("Failed to send request.");
    check_response_assets(&response, 100);

    let user_account_id = AccountId::new("root".into(), "global");
    let get_user_account = by_id(user_account_id);
    let response = iroha_client
        .request(&get_user_account)
        .await
        .expect("Failed to send request.");
    check_response_assets(&response, 0);

    let substrate_acc = AccountId32::decode(&mut &([0x34, 0x2d, 0x54, 0x43, 0x89, 0x54, 0x2f, 0xfc, 0x23, 0x3b, 0xed, 0x2c, 0x90, 0x46, 0x47, 0xce, 0xf3, 0x43, 0x08, 0x73, 0xf7, 0xbd, 0xcc, 0x1a, 0xb5, 0xe2, 0xe8, 0x51, 0x7b, 0x0c, 0x51, 0x78])[..]).unwrap();
    let xor_storage_key: sp_core::storage::StorageKey = api
        .metadata
        .storage_map_key::<AccountId32, AccountData>("XOR", "Account", substrate_acc.clone())
        .unwrap();

    let balance = api.get_storage_by_key_hash::<AccountData>(xor_storage_key.clone(), None).map(|x| x.free / 1000).unwrap_or(0);
    println!("[BRIDGE TEST] root@global account balance on Substrate is: {} XOR", balance);
    assert_eq!(balance, 0);

    // send transaction to substrate to handle the transfer
    println!("[BRIDGE TEST] Sending transaction to substrate...");
    let tx_hash = api
        .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
        .unwrap();
    println!(
        "[BRIDGE TEST] Transaction got finalized. Hash: {:?}\n",
        tx_hash
    );

    async_std::task::sleep(std::time::Duration::from_secs(3)).await;

    println!("[BRIDGE TEST] Checking account balances after the transfer...");
    // check assets after the transfer
    let response = iroha_client
        .request(&get_bridge_account)
        .await
        .expect("Failed to send request.");
    check_response_assets(&response, 100);

    let response = iroha_client
        .request(&get_user_account)
        .await
        .expect("Failed to send request.");
    check_response_assets(&response,  0);

    let balance = api.get_storage_by_key_hash::<AccountData>(xor_storage_key, None).map(|x| x.free / 1000).unwrap();
    println!("[BRIDGE TEST] root@global account balance on Substrate is: {} XOR", balance);
    assert_eq!(balance, 100);

    println!("[BRIDGE TEST] Test passed!");
}
