use codec::{Decode, Encode};
use ed25519_dalek::Keypair;
use keyring::ed25519::{ed25519::Pair as EdPair, Keyring};
use keyring::sr25519::sr25519::Pair as SrPair;
// use schnorrkel::SecretKey;

use iroha::prelude::*;
use iroha_client::{
    client::{account::by_id, Client},
    config::Configuration,
};
use sp_core::crypto::Pair;
use sp_runtime::AccountId32;

use iroha::asset::query::GetAccountAssetsResult;
use iroha::permission::Permission::Anything;
use iroha::permission::Permissions;
use sp_core::crypto::Ss58Codec;
use substrate_api_client::{
    compose_extrinsic, extrinsic::xt_primitives::UncheckedExtrinsicV4, utils::hexstr_to_vec,
    AccountData, AccountInfo, Api, XtStatus,
};

fn check_response_assets(response: &QueryResult, expected_xor_amount: u32) {
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

    // "PUBLIC_KEY": {"inner": [52, 45, 84, 67, 137, 84, 47, 252, 35, 59, 237, 44, 144, 70, 71, 206, 243, 67, 8, 115, 247, 189, 204, 26, 181, 226, 232, 81, 123, 12, 81, 120]},
    let acc_pk = [
        0x34, 0x2d, 0x54, 0x43, 0x89, 0x54, 0x2f, 0xfc, 0x23, 0x3b, 0xed, 0x2c, 0x90, 0x46, 0x47,
        0xce, 0xf3, 0x43, 0x08, 0x73, 0xf7, 0xbd, 0xcc, 0x1a, 0xb5, 0xe2, 0xe8, 0x51, 0x7b, 0x0c,
        0x51, 0x78,
    ];
    let substrate_acc = AccountId32::decode(&mut &(acc_pk)[..]).unwrap();
    let xor_storage_key: sp_core::storage::StorageKey = api
        .metadata
        .storage_map_key::<AccountId32, AccountData>("XOR", "Account", substrate_acc.clone())
        .unwrap();

    let balance = api
        .get_storage_by_key_hash::<AccountData>(xor_storage_key.clone(), None)
        .map(|x| x.free / 1000)
        .unwrap_or(0);
    println!(
        "[BRIDGE TEST] root@global account balance on Substrate is: {} XOR",
        balance
    );
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
    check_response_assets(&response, 0);

    let balance = api
        .get_storage_by_key_hash::<AccountData>(xor_storage_key, None)
        .map(|x| x.free / 1000)
        .unwrap();
    println!(
        "[BRIDGE TEST] root@global account balance on Substrate is: {} XOR",
        balance
    );
    assert_eq!(balance, 100);

    // test incoming transfer
    let sk = [
        18, 182, 246, 209, 68, 27, 219, 111, 25, 143, 14, 178, 64, 212, 107, 38, 113, 40, 79, 226,
        81, 217, 198, 102, 12, 68, 238, 115, 162, 63, 242, 255, 52, 45, 84, 67, 137, 84, 47, 252,
        35, 59, 237, 44, 144, 70, 71, 206, 243, 67, 8, 115, 247, 189, 204, 26, 181, 226, 232, 81,
        123, 12, 81, 120,
    ];
    let root_kp = EdPair::from_seed_slice(&sk[..32]).unwrap();
    // let api = Api::new(format!("ws://{}", url)).set_signer(root_kp.clone());
    let root_acc = root_kp.public();
    let amount = 100u128;
    let request_transfer: UncheckedExtrinsicV4<_> = compose_extrinsic!(
        api.clone(),
        "TemplateModule",
        "request_transfer",
        substrate_acc,
        amount * 1000,
        0u8
    );
    let tx_hash = api
        .send_extrinsic(request_transfer.hex_encode(), XtStatus::InBlock)
        .unwrap();
    println!(
        "[BRIDGE TEST] Transaction got finalized. Hash: {:?}\n",
        tx_hash
    );

    async_std::task::sleep(std::time::Duration::from_secs(3)).await;

    let response = iroha_client
        .request(&get_user_account)
        .await
        .expect("Failed to send request.");
    check_response_assets(&response, 100);

    println!("[BRIDGE TEST] Test passed!");
}
