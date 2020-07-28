use codec::{Decode, Encode};
use keyring::ed25519::{ed25519::Pair as EdPair, Keyring};
use keyring::sr25519::sr25519::Pair as SrPair;

use iroha::prelude::*;
use iroha_client::{
    client::{assets::by_account_id, Client},
    config::Configuration,
};
use sp_core::crypto::Pair;

use iroha::asset::query::GetAccountAssetsResult;
use iroha::permission::Permission::Anything;
use iroha::permission::Permissions;
use substrate_api_client::{
    compose_extrinsic, extrinsic::xt_primitives::UncheckedExtrinsicV4, utils::hexstr_to_vec, Api,
    XtStatus,
};

fn check_response_assets(
    response: &QueryResult,
    expected_dot_amount: u32,
    expected_xor_amount: u32,
) {
    if let QueryResult::GetAccount(get_account_result) = response {
        let account = &get_account_result.account;
        let assets = &account.assets;
        let dot_amount = assets
            .iter()
            .find(|(_, asset)| asset.id.definition_id.name == "DOT")
            .map(|asset| asset.quantity)
            .unwrap_or(0);
        let xor_amount = assets
            .iter()
            .find(|(_, asset)| asset.id.definition_id.name == "XOR")
            .map(|asset| asset.quantity)
            .unwrap_or(0);
        assert_eq!(dot_amount, expected_dot_amount);
        assert_eq!(xor_amount, expected_xor_amount);
        println!("{} account balance is: DOT: {}, XOR: {}", account.id, expected_dot_amount, expected_xor_amount);
    } else {
        panic!("Test failed.");
    }
}

#[async_std::main]
async fn main() {
    let configuration =
        Configuration::from_path("config.json").expect("Failed to load configuration.");
    let mut iroha_client = Client::new(&configuration);

    println!("Checking account balances before the swap...");
    // check assets before the swap, but after the user sent transaction to the bridge
    let bridge_account_id = AccountId::new("bridge", "polkadot");
    let get_bridge_account = by_account_id(bridge_account_id);
    let response = iroha_client
        .request(&get_bridge_account)
        .await
        .expect("Failed to send request.");
    check_response_assets(&response, 0, 100);

    let user_account_id = AccountId::new("root".into(), "global");
    let get_user_account = by_account_id(user_account_id);
    let response = iroha_client
        .request(&get_user_account)
        .await
        .expect("Failed to send request.");
    check_response_assets(&response, 0, 0);

    // send transaction to substrate to handle the transfer
    let url = "127.0.0.1:9944";
    let seed = "Alice";
    let signer = SrPair::from_string(&format!("//{}", seed), None).unwrap();
    let api = Api::new(format!("ws://{}", url)).set_signer(signer);
    let xt: UncheckedExtrinsicV4<_> =
        compose_extrinsic!(api.clone(), "TemplateModule", "fetch_blocks_signed");

    println!("Sending transaction to substrate...");
    let tx_hash = api
        .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
        .unwrap();
    println!("Transaction got finalized. Hash: {:?}\n", tx_hash);

    println!("Waiting for all Iroha transactions to confirm...");
    async_std::task::sleep(std::time::Duration::from_secs(5)).await;

    println!("Checking account balances after the swap...");
    // check assets after the swap
    let response = iroha_client
        .request(&get_bridge_account)
        .await
        .expect("Failed to send request.");
    check_response_assets(&response, 0, 100);

    let response = iroha_client
        .request(&get_user_account)
        .await
        .expect("Failed to send request.");
    check_response_assets(&response, 200, 0);

    println!("Test passed!");
}
