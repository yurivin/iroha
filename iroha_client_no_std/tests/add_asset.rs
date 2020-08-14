#[cfg(test)]
mod tests {
    use async_std::task;
    use iroha::{config::Configuration, Iroha};
    use iroha_client_no_std::{
        asset::isi::AssetInstruction,
        client::{self, Client},
        config::Configuration as ClientConfiguration,
        domain::isi::DomainInstruction,
        isi,
        prelude::*,
    };
    use std::{thread, time::Duration};
    use tempfile::TempDir;

    const CONFIGURATION_PATH: &str = "tests/test_config.json";

    #[async_std::test]
    //TODO: use cucumber to write `gherkin` instead of code.
    async fn client_add_asset_quantity_to_existing_asset_should_increase_asset_amount() {
        // Given
        thread::spawn(create_and_start_iroha);
        thread::sleep(std::time::Duration::from_millis(300));
        let configuration =
            Configuration::from_path(CONFIGURATION_PATH).expect("Failed to load configuration.");
        let domain_name = "global";
        let account_name = "root";
        let account_id = AccountId::new(account_name, domain_name);
        let asset_definition_id = AssetDefinitionId::new("xor", domain_name);
        let create_asset = isi::Instruction::Domain(DomainInstruction::RegisterAsset(
            domain_name.to_string(),
            AssetDefinition::new(asset_definition_id.clone()),
        ));
        let mut iroha_client = Client::new(&ClientConfiguration::from_iroha_configuration(
            &configuration,
        ));
        iroha_client
            .submit(create_asset.into())
            .await
            .expect("Failed to prepare state.");
        task::sleep(Duration::from_millis(
            &configuration.sumeragi_configuration.pipeline_time_ms() * 2,
        ))
        .await;
        //When
        let quantity: u32 = 200;
        let mint_asset = isi::Instruction::Asset(AssetInstruction::MintAsset(
            quantity,
            AssetId {
                definition_id: asset_definition_id.clone(),
                account_id: account_id.clone(),
            },
        ));
        iroha_client
            .submit(mint_asset.into())
            .await
            .expect("Failed to create asset.");
        task::sleep(Duration::from_millis(
            &configuration.sumeragi_configuration.pipeline_time_ms() * 2,
        ))
        .await;
        //Then
        let request = client::assets::by_account_id(account_id);
        let query_result = iroha_client
            .request(&request)
            .await
            .expect("Failed to execute request.");
        if let QueryResult::GetAccountAssets(result) = query_result {
            assert!(!result.assets.is_empty());
            assert_eq!(
                quantity,
                result.assets.last().expect("Asset should exist.").quantity,
            );
        } else {
            panic!("Wrong Query Result Type.");
        }
    }

    fn create_and_start_iroha() {
        let temp_dir = TempDir::new().expect("Failed to create TempDir.");
        let mut configuration =
            Configuration::from_path(CONFIGURATION_PATH).expect("Failed to load configuration.");
        configuration
            .kura_configuration
            .kura_block_store_path(temp_dir.path());
        let iroha = Iroha::new(configuration);
        task::block_on(iroha.start()).expect("Failed to start Iroha.");
        //Prevents temp_dir from clean up untill the end of the tests.
        #[allow(clippy::empty_loop)]
        loop {}
    }
}
