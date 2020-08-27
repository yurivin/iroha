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

#[derive(Encode, Decode, Clone, Copy, Debug)]
pub enum AssetKind {
    XOR,
    DOT,
    KSM,
}

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

    // check assets before the transfer
    println!("[BRIDGE TEST] Checking account balances before the Iroha->Bridge transfer...");
    let bridge_account_id = AccountId::new("bridge", "polkadot");
    let get_bridge_account = by_id(bridge_account_id.clone());
    let response = iroha_client
        .request(&get_bridge_account)
        .await
        .expect("Failed to send request.");
    check_response_assets(&response, 0);

    let global_domain_name = "global";
    let user_account_id = AccountId::new("root".into(), global_domain_name);
    let get_user_account = by_id(user_account_id.clone());
    let response = iroha_client
        .request(&get_user_account)
        .await
        .expect("Failed to send request.");
    check_response_assets(&response, 100);

    let substrate_user_account =
        AccountId32::decode(&mut &configuration.public_key.inner[..]).unwrap();
    let xor_storage_key: sp_core::storage::StorageKey = api
        .metadata
        .storage_map_key::<AccountId32, AccountData>(
            "XOR",
            "Account",
            substrate_user_account.clone(),
        )
        .unwrap();
    let balance = api
        .get_storage_by_key_hash::<AccountData>(xor_storage_key.clone(), None)
        .map(|x| x.free)
        .unwrap_or(0);
    println!(
        "[BRIDGE TEST] root@global account balance on Substrate is: {} XOR",
        balance
    );
    assert_eq!(balance, 0);

    let xor_asset_def = AssetDefinition::new(AssetDefinitionId {
        name: "XOR".into(),
        domain_name: global_domain_name.into(),
    });
    let iroha_transfer_xor = Transfer::new(
        user_account_id.clone(),
        Asset::with_quantity(
            AssetId::new(xor_asset_def.id.clone(), user_account_id.clone()),
            100,
        ),
        bridge_account_id.clone(),
    )
    .into();
    iroha_client
        .submit(iroha_transfer_xor)
        .await
        .expect("Failed to send request");
    println!("[BRIDGE TEST] Sent Iroha->Bridge transfer transaction.");

    async_std::task::sleep(std::time::Duration::from_secs(15)).await;
    println!("[BRIDGE TEST] Checking account balances after the Iroha->Bridge transfer...");

    let get_user_account = by_id(user_account_id.clone());
    let response = iroha_client
        .request(&get_user_account)
        .await
        .expect("Failed to send request.");
    check_response_assets(&response, 0);

    let balance = api
        .get_storage_by_key_hash::<AccountData>(xor_storage_key.clone(), None)
        .map(|x| x.free)
        .unwrap_or(0);
    println!(
        "[BRIDGE TEST] root@global account balance on Substrate is: {} XOR",
        balance
    );
    assert_eq!(balance, 100);

    // test incoming transfer
    let amount = 100u128;
    let nonce = 0u8;

    let signer = EdPair::from_seed(&[
        18, 182, 246, 209, 68, 27, 219, 111, 25, 143, 14, 178, 64, 212, 107, 38, 113, 40, 79, 226,
        81, 217, 198, 102, 12, 68, 238, 115, 162, 63, 242, 255,
    ]);
    let api = Api::new(format!("ws://{}", url)).set_signer(signer);
    let request_transfer: UncheckedExtrinsicV4<_> = compose_extrinsic!(
        api.clone(),
        "IrohaBridge",
        "request_transfer",
        user_account_id.clone(),
        AssetKind::XOR,
        amount,
        nonce
    );
    println!("[BRIDGE TEST] Sent Bridge->Iroha transfer transaction.");
    let tx_hash = api
        .send_extrinsic(request_transfer.hex_encode(), XtStatus::Finalized)
        .unwrap();
    println!(
        "[BRIDGE TEST] Transaction got finalized. Hash: {:?}\n",
        tx_hash
    );

    let response = iroha_client
        .request(&get_user_account)
        .await
        .expect("Failed to send request.");
    check_response_assets(&response, 100);

    let balance = api
        .get_storage_by_key_hash::<AccountData>(xor_storage_key.clone(), None)
        .map(|x| x.free)
        .unwrap();
    println!(
        "[BRIDGE TEST] root@global account balance on Substrate is: {} XOR",
        balance
    );
    assert_eq!(balance, 0);

    println!("[BRIDGE TEST] Test passed!");
}
