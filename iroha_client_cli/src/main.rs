use clap::{App, Arg};
use iroha_client::config::Configuration;

const CONFIG: &str = "config";
const DOMAIN: &str = "domain";
const ACCOUNT: &str = "account";
const ASSET: &str = "asset";
const MAINTENANCE: &str = "maintenance";
const DEX: &str = "dex";

fn main() {
    let matches = App::new("Iroha CLI Client")
        .version("0.1.0")
        .author("Nikita Puzankov <puzankov@soramitsu.co.jp>")
        .about("Iroha CLI Client provides an ability to interact with Iroha Peers Web API without direct network usage.")
        .arg(
            Arg::with_name(CONFIG)
            .short("c")
            .long(CONFIG)
            .value_name("FILE")
            .help("Sets a config file path.")
            .takes_value(true)
            .default_value("config.json"),
            )
        .subcommand(
            domain::build_app(),
            )
        .subcommand(
            account::build_app(),
            )
        .subcommand(
            asset::build_app(),
            )
        .subcommand(
            maintenance::build_app(),
            )
        .subcommand(
            dex::build_app(),
        )
        .get_matches();
    let configuration_path = matches
        .value_of(CONFIG)
        .expect("Failed to get configuration path.");
    println!("Value for config: {}", configuration_path);
    let configuration =
        Configuration::from_path(configuration_path).expect("Failed to load configuration");
    if let Some(ref matches) = matches.subcommand_matches(DOMAIN) {
        domain::process(matches, &configuration);
    }
    if let Some(ref matches) = matches.subcommand_matches(ACCOUNT) {
        account::process(matches, &configuration);
    }
    if let Some(ref matches) = matches.subcommand_matches(ASSET) {
        asset::process(matches, &configuration);
    }
    if let Some(ref matches) = matches.subcommand_matches(MAINTENANCE) {
        maintenance::process(matches, &configuration);
    }
    if let Some(ref matches) = matches.subcommand_matches(DEX) {
        dex::process(matches);
    }
}

mod domain {
    use super::*;
    use clap::ArgMatches;
    use futures::executor;
    use iroha::{isi, peer::PeerId, prelude::*};
    use iroha_client::{client::Client, config::Configuration};

    const DOMAIN_NAME: &str = "name";
    const ADD: &str = "add";

    pub fn build_app<'a, 'b>() -> App<'a, 'b> {
        App::new(DOMAIN)
            .about("Use this command to work with Domain Entities in Iroha Peer.")
            .subcommand(
                App::new(ADD).arg(
                    Arg::with_name(DOMAIN_NAME)
                        .long(DOMAIN_NAME)
                        .value_name(DOMAIN_NAME)
                        .help("Domain's name as double-quoted string.")
                        .takes_value(true)
                        .required(true),
                ),
            )
    }

    pub fn process(matches: &ArgMatches<'_>, configuration: &Configuration) {
        if let Some(ref matches) = matches.subcommand_matches(ADD) {
            if let Some(domain_name) = matches.value_of(DOMAIN_NAME) {
                println!("Adding a new Domain with a name: {}", domain_name);
                create_domain(domain_name, configuration);
            }
        }
    }

    fn create_domain(domain_name: &str, configuration: &Configuration) {
        let mut iroha_client = Client::new(configuration);
        let create_domain = isi::Add {
            object: Domain::new(domain_name.to_string()),
            destination_id: PeerId::new(&configuration.torii_url, &configuration.public_key),
        };
        executor::block_on(iroha_client.submit(create_domain.into()))
            .expect("Failed to create domain.");
    }
}

mod account {
    use super::*;
    use clap::ArgMatches;
    use futures::executor;
    use iroha::account::query;
    use iroha::dex::isi::*;
    use iroha::{isi, prelude::*};
    use iroha_client::{
        client::{util, Client},
        config::Configuration,
    };

    const REGISTER: &str = "register";
    const ACCOUNT_NAME: &str = "name";
    const ACCOUNT_DOMAIN_NAME: &str = "domain";
    const ACCOUNT_KEY: &str = "key";
    const GET: &str = "get";
    const ADD_TRANSFER_PERMISSION: &str = "add_transfer_permission";
    const ACCOUNT_ID: &str = "id";
    const ASSET_DEFINITION_ID: &str = "asset_id";

    pub fn build_app<'a, 'b>() -> App<'a, 'b> {
        App::new(ACCOUNT)
            .about("Use this command to work with Account Entities in Iroha Peer.")
            .subcommand(
                App::new(REGISTER)
                    .about("Use this command to register new Account in existing Iroha Domain.")
                    .arg(
                        Arg::with_name(ACCOUNT_NAME)
                            .long(ACCOUNT_NAME)
                            .value_name(ACCOUNT_NAME)
                            .help("Account's name as double-quoted string.")
                            .takes_value(true)
                            .required(true),
                    )
                    .arg(
                        Arg::with_name(ACCOUNT_DOMAIN_NAME)
                            .long(ACCOUNT_DOMAIN_NAME)
                            .value_name(ACCOUNT_DOMAIN_NAME)
                            .help("Account's Domain's name as double-quoted string.")
                            .takes_value(true)
                            .required(true),
                    )
                    .arg(
                        Arg::with_name(ACCOUNT_KEY)
                            .long(ACCOUNT_KEY)
                            .value_name(ACCOUNT_KEY)
                            .help("Account's public key as double-quoted string.")
                            .takes_value(true)
                            .required(true),
                    ),
            )
            .subcommand(
                App::new(GET)
                    .about("Use this command to get existing Account information.")
                    .arg(
                        Arg::with_name(ACCOUNT_NAME)
                            .long(ACCOUNT_NAME)
                            .value_name(ACCOUNT_NAME)
                            .help("Account's name as double-quoted string.")
                            .takes_value(true)
                            .required(true),
                    )
                    .arg(
                        Arg::with_name(ACCOUNT_DOMAIN_NAME)
                            .long(ACCOUNT_DOMAIN_NAME)
                            .value_name(ACCOUNT_DOMAIN_NAME)
                            .help("Account's Domain's name as double-quoted string.")
                            .takes_value(true)
                            .required(true),
                    ),
            )
            .subcommand(
                App::new(ADD_TRANSFER_PERMISSION)
                    .about("Use this command to add TrasferAsset permission for account.")
                    .arg(
                        Arg::with_name(ACCOUNT_ID)
                            .long(ACCOUNT_ID)
                            .value_name(ACCOUNT_ID)
                            .help("Account's Id as double-quoted string in the format `account_name@domain_name`.")
                            .takes_value(true)
                            .required(true),
                    )
                    .arg(
                        Arg::with_name(ASSET_DEFINITION_ID)
                            .long(ASSET_DEFINITION_ID)
                            .value_name(ASSET_DEFINITION_ID)
                            .help("Asset's Id as double-quoted string in the following format `asset_name#domain_name`.")
                            .takes_value(true)
                            .required(true),
                    ),
            )
    }

    pub fn process(matches: &ArgMatches<'_>, configuration: &Configuration) {
        if let Some(ref matches) = matches.subcommand_matches(REGISTER) {
            if let Some(account_name) = matches.value_of(ACCOUNT_NAME) {
                println!("Creating account with a name: {}", account_name);
                if let Some(domain_name) = matches.value_of(ACCOUNT_DOMAIN_NAME) {
                    println!("Creating account with a domain's name: {}", domain_name);
                    if let Some(public_key) = matches.value_of(ACCOUNT_KEY) {
                        println!("Creating account with a public key: {}", public_key);
                        create_account(account_name, domain_name, public_key, configuration);
                    }
                }
            }
        }
        if let Some(ref matches) = matches.subcommand_matches(GET) {
            if let Some(account_name) = matches.value_of(ACCOUNT_NAME) {
                println!("Getting account with a name: {}", account_name);
                if let Some(domain_name) = matches.value_of(ACCOUNT_DOMAIN_NAME) {
                    println!("Getting account with a domain's name: {}", domain_name);
                    get_account(account_name, domain_name);
                }
            }
        }
        if let Some(ref matches) = matches.subcommand_matches(ADD_TRANSFER_PERMISSION) {
            if let Some(account_id) = matches.value_of(ACCOUNT_ID) {
                if let Some(asset_definition_id) = matches.value_of(ASSET_DEFINITION_ID) {
                    println!(
                        "Adding transfer permission for account: {} to transfer asset: {}",
                        account_id, asset_definition_id
                    );
                    add_transfer_permission(asset_definition_id, account_id);
                }
            }
        }
    }

    fn create_account(
        account_name: &str,
        domain_name: &str,
        public_key: &str,
        configuration: &Configuration,
    ) {
        let public_key = util::public_key_from_str(public_key).unwrap();
        let create_account = isi::Register {
            object: Account::with_signatory(account_name, domain_name, public_key),
            destination_id: String::from(domain_name),
        };
        let mut iroha_client = Client::new(configuration);
        executor::block_on(iroha_client.submit(create_account.into()))
            .expect("Failed to create account.");
    }

    fn get_account(account_name: &str, domain_name: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let account_id = AccountId::new(account_name, domain_name);
        let query_result =
            executor::block_on(iroha_client.request(&query::GetAccount::build_request(account_id)))
                .expect("Failed to get account information.");
        if let QueryResult::GetAccount(result) = query_result {
            println!("Get Account information result: {:#?}", result);
        }
    }

    fn add_transfer_permission(asset_id: &str, account_id: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let account_id = AccountId::from(account_id);
        let asset_definition_id = AssetDefinitionId::from(asset_id);
        let instruction = add_transfer_permission_for_account(asset_definition_id, account_id);
        executor::block_on(iroha_client.submit(instruction))
            .expect("Failed to add TransferAsset permission for account")
    }
}

mod asset {
    use super::*;
    use clap::ArgMatches;
    use futures::executor;
    use iroha::{isi, prelude::*};
    use iroha_client::{
        client::{self, Client},
        config::Configuration,
    };

    const REGISTER: &str = "register";
    const MINT: &str = "mint";
    const GET: &str = "get";
    const ASSET_NAME: &str = "name";
    const ASSET_DOMAIN_NAME: &str = "domain";
    const ASSET_ACCOUNT_ID: &str = "account_id";
    const ASSET_ID: &str = "id";
    const QUANTITY: &str = "quantity";

    pub fn build_app<'a, 'b>() -> App<'a, 'b> {
        App::new(ASSET)
            .about("Use this command to work with Asset and Asset Definition Entities in Iroha Peer.")
            .subcommand(
                App::new(REGISTER)
                .about("Use this command to register new Asset Definition in existing Iroha Domain.")
                .arg(
                    Arg::with_name(ASSET_DOMAIN_NAME)
                    .long(ASSET_DOMAIN_NAME)
                    .value_name(ASSET_DOMAIN_NAME)
                    .help("Asset's domain's name as double-quoted string.")
                    .takes_value(true)
                    .required(true),
                    )
                .arg(
                    Arg::with_name(ASSET_NAME)
                    .long(ASSET_NAME)
                    .value_name(ASSET_NAME)
                    .help("Asset's name as double-quoted string.")
                    .takes_value(true)
                    .required(true),
                    )
                )
            .subcommand(
                App::new(MINT)
                .about("Use this command to Mint Asset in existing Iroha Account.")
                .arg(Arg::with_name(ASSET_ACCOUNT_ID).long(ASSET_ACCOUNT_ID).value_name(ASSET_ACCOUNT_ID).help("Account's id as double-quoted string in the following format `account_name@domain_name`.")
                .takes_value(true)
                .required(true)
                )
            .arg(
                    Arg::with_name(ASSET_ID)
                        .long(ASSET_ID)
                        .value_name(ASSET_ID)
                        .help("Asset's id as double-quoted string in the following format `asset_name#domain_name`.")
                        .takes_value(true)
                        .required(true)
                )
                .arg(
                    Arg::with_name(QUANTITY)
                        .long(QUANTITY)
                        .value_name(QUANTITY)
                        .help("Asset's quantity as a number.")
                        .takes_value(true)
                        .required(true)
                    )
            )
            .subcommand(
                App::new(GET)
                .about("Use this command to get Asset information from Iroha Account.")
                .arg(Arg::with_name(ASSET_ACCOUNT_ID).long(ASSET_ACCOUNT_ID).value_name(ASSET_ACCOUNT_ID).help("Account's id as double-quoted string in the following format `account_name@domain_name`.").takes_value(true).required(true)
                )
                    .arg(
                        Arg::with_name(ASSET_ID)
                            .long(ASSET_ID)
                            .value_name(ASSET_ID)
                            .help("Asset's id as double-quoted string in the following format `asset_name#domain_name`.")
                            .takes_value(true)
                            .required(true)
                    )
                )
    }

    pub fn process(matches: &ArgMatches<'_>, configuration: &Configuration) {
        if let Some(ref matches) = matches.subcommand_matches(REGISTER) {
            if let Some(asset_name) = matches.value_of(ASSET_NAME) {
                println!("Registering asset defintion with a name: {}", asset_name);
                if let Some(domain_name) = matches.value_of(ASSET_DOMAIN_NAME) {
                    println!(
                        "Registering asset definition with a domain's name: {}",
                        domain_name
                    );
                    register_asset_definition(asset_name, domain_name, configuration);
                }
            }
        }
        if let Some(ref matches) = matches.subcommand_matches(MINT) {
            if let Some(asset_id) = matches.value_of(ASSET_ID) {
                println!("Minting asset with an identification: {}", asset_id);
                if let Some(account_id) = matches.value_of(ASSET_ACCOUNT_ID) {
                    println!(
                        "Minting asset to account with an identification: {}",
                        account_id
                    );
                    if let Some(amount) = matches.value_of(QUANTITY) {
                        println!("Minting asset's quantity: {}", amount);
                        mint_asset(asset_id, account_id, amount, configuration);
                    }
                }
            }
        }
        if let Some(ref matches) = matches.subcommand_matches(GET) {
            if let Some(asset_id) = matches.value_of(ASSET_ID) {
                println!("Getting asset with an identification: {}", asset_id);
                if let Some(account_id) = matches.value_of(ASSET_ACCOUNT_ID) {
                    println!("Getting account with an identification: {}", account_id);
                    get_asset(asset_id, account_id, configuration);
                }
            }
        }
    }

    fn register_asset_definition(
        asset_name: &str,
        domain_name: &str,
        configuration: &Configuration,
    ) {
        let mut iroha_client = Client::new(configuration);
        executor::block_on(
            iroha_client.submit(
                isi::Register {
                    object: AssetDefinition::new(AssetDefinitionId::new(asset_name, domain_name)),
                    destination_id: domain_name.to_string(),
                }
                .into(),
            ),
        )
        .expect("Failed to create account.");
    }

    fn mint_asset(
        asset_definition_id: &str,
        account_id: &str,
        quantity: &str,
        configuration: &Configuration,
    ) {
        let quantity: u32 = quantity.parse().expect("Failed to parse Asset quantity.");
        let mint_asset = isi::Mint {
            object: quantity,
            destination_id: AssetId {
                definition_id: AssetDefinitionId::from(asset_definition_id),
                account_id: AccountId::from(account_id),
            },
        };
        let mut iroha_client = Client::new(configuration);
        executor::block_on(iroha_client.submit(mint_asset.into()))
            .expect("Failed to create account.");
    }

    fn get_asset(_asset_id: &str, account_id: &str, configuration: &Configuration) {
        let mut iroha_client = Client::new(configuration);
        let query_result = executor::block_on(iroha_client.request(
            &client::assets::by_account_id(<Account as Identifiable>::Id::from(account_id)),
        ))
        .expect("Failed to get asset.");
        if let QueryResult::GetAccountAssets(result) = query_result {
            println!("Get Asset result: {:#?}", result);
        }
    }
    #[cfg(test)]
    mod tests {
        use async_std::task;
        use iroha::{config::Configuration, isi, prelude::*};
        use iroha_client::{client::Client, config::Configuration as ClientConfiguration};
        use std::time::Duration;
        use tempfile::TempDir;

        const CONFIGURATION_PATH: &str = "tests/test_config.json";

        #[async_std::test]
        async fn cli_check_health_should_work() {
            task::spawn(async {
                let temp_dir = TempDir::new().expect("Failed to create TempDir.");
                let mut configuration = Configuration::from_path(CONFIGURATION_PATH)
                    .expect("Failed to load configuration.");
                configuration
                    .kura_configuration
                    .kura_block_store_path(temp_dir.path());
                let iroha = Iroha::new(configuration.clone());
                iroha.start().await.expect("Failed to start Iroha.");
                //Prevents temp_dir from clean up untill the end of the tests.
                #[allow(clippy::empty_loop)]
                loop {}
            });
            task::sleep(Duration::from_millis(300)).await;
            super::health();
        }

        #[async_std::test]
        async fn cli_scrape_metrics_should_work() {
            task::spawn(async {
                let temp_dir = TempDir::new().expect("Failed to create TempDir.");
                let mut configuration = Configuration::from_path(CONFIGURATION_PATH)
                    .expect("Failed to load configuration.");
                configuration
                    .kura_configuration
                    .kura_block_store_path(temp_dir.path());
                let iroha = Iroha::new(configuration.clone());
                iroha.start().await.expect("Failed to start Iroha.");
                //Prevents temp_dir from clean up untill the end of the tests.
                #[allow(clippy::empty_loop)]
                loop {}
            });
            task::sleep(Duration::from_millis(300)).await;
            super::metrics();
        }

        #[async_std::test]
        async fn cli_connect_to_consume_block_changes_should_work() {
            task::spawn(async {
                let temp_dir = TempDir::new().expect("Failed to create TempDir.");
                let mut configuration = Configuration::from_path(CONFIGURATION_PATH)
                    .expect("Failed to load configuration.");
                configuration
                    .kura_configuration
                    .kura_block_store_path(temp_dir.path());
                let iroha = Iroha::new(configuration.clone());
                iroha.start().await.expect("Failed to start Iroha.");
                //Prevents temp_dir from clean up untill the end of the tests.
                #[allow(clippy::empty_loop)]
                loop {}
            });
            task::sleep(Duration::from_millis(300)).await;
            let connection_future = async_std::future::timeout(
                Duration::from_millis(300),
                task::spawn(async { super::connect("transaction", "all") }),
            );
            let domain_name = "global";
            let asset_definition_id = AssetDefinitionId::new("xor", domain_name);
            let create_asset = isi::Register {
                object: AssetDefinition::new(asset_definition_id),
                destination_id: domain_name.to_string(),
            };
            let mut iroha_client = Client::new(&ClientConfiguration::from_iroha_configuration(
                &Configuration::from_path(CONFIGURATION_PATH)
                    .expect("Failed to load configuration."),
            ));
            iroha_client
                .submit(create_asset.into())
                .await
                .expect("Failed to prepare state.");
            if let Ok(result) = connection_future.await {
                result.expect("Failed to connect.")
            }
        }
    }
}

mod dex {
    use super::*;
    use clap::ArgMatches;
    use futures::executor;
    use iroha::dex::isi::*;
    use iroha::dex::query::*;
    use iroha::dex::*;
    use iroha::{isi, prelude::*};
    use iroha_client::{
        client::{util, Client},
        config::Configuration,
    };

    const INITIALIZE: &str = "initialize";
    const TOKEN_PAIR: &str = "token_pair";
    const OWNER_ACCOUNT_ID: &str = "owner_account_id";
    const BASE_ASSET_ID: &str = "base_asset_id";
    const TARGET_ASSET_ID: &str = "target_asset_id";
    const CREATE: &str = "create";
    const REMOVE: &str = "remove";
    const LIST: &str = "list";
    const GET: &str = "get";
    const DOMAIN_NAME: &str = "domain";
    const XYK_POOL: &str = "xyk_pool";
    const BASE_AMOUNT: &str = "base_amount";
    const TARGET_AMOUNT: &str = "target_amount";
    const INPUT_AMOUNT: &str = "input_amount";
    const OUTPUT_AMOUNT: &str = "output_amount";
    const LIQUIDITY: &str = "liquidity";
    const ADD_LIQUIDITY: &str = "add_liquidity";
    const REMOVE_LIQUIDITY: &str = "remove_liquidity";
    const XYK_POOL_SWAP: &str = "xyk_pool_swap";
    const PATH: &str = "path";

    pub fn build_app<'a, 'b>() -> App<'a, 'b> {
        App::new(DEX)
            .about("Use this command to work with DEX Entities in Iroha Peer.")
            .subcommand(
                App::new(INITIALIZE)
                    .about("Use this command to initialize the DEX in existing Iroha Domain.")
                    .arg(
                        Arg::with_name(DOMAIN_NAME)
                            .long(DOMAIN_NAME)
                            .value_name(DOMAIN_NAME)
                            .help("DEX's Domain's name as double-quoted string.")
                            .takes_value(true)
                            .required(true)
                    )
                    .arg(
                        Arg::with_name(OWNER_ACCOUNT_ID)
                            .long(OWNER_ACCOUNT_ID)
                            .value_name(OWNER_ACCOUNT_ID)
                            .help("DEX Owner Account's id as double-quoted string in the following format `account_name@domain_name`.")
                            .takes_value(true)
                            .required(true)
                    )
                    .arg(
                        Arg::with_name(BASE_ASSET_ID)
                            .long(BASE_ASSET_ID)
                            .value_name(BASE_ASSET_ID)
                            .help("Base Asset Id as double-quoted string in the following format `asset_name#domain_name`.")
                            .takes_value(true)
                            .required(true)
                    )
            )
            .subcommand(
                App::new(LIST)
                    .about("Use this command to list all active DEX in Iroha Peer.")
            )
            .subcommand(
                App::new(GET)
                    .about("Use this command to get particular DEX information.")
                    .arg(
                        Arg::with_name(DOMAIN_NAME)
                            .long(DOMAIN_NAME)
                            .value_name(DOMAIN_NAME)
                            .help("DEX's domain's name as double-quoted string.")
                            .takes_value(true)
                            .required(true)
                    )
            )
            .subcommand(
                App::new(TOKEN_PAIR)
                    .about("Use this command to work with TokenPair Entities in active DEX.")
                    .arg(Arg::with_name(DOMAIN_NAME).long(DOMAIN_NAME).value_name(DOMAIN_NAME).help("DEX's domain's name as double-quoted string.").takes_value(true).required(true))
                    .subcommand(
                        App::new(CREATE)
                            .about("Use this command to create a new Token Pair for existing Assets.")
                            .arg(
                                Arg::with_name(BASE_ASSET_ID)
                                    .long(BASE_ASSET_ID)
                                    .value_name(BASE_ASSET_ID)
                                    .help("Base Asset's Id as double-quoted string in the following format `asset_name#domain_name`.")
                                    .takes_value(true)
                                    .required(true)
                            )
                            .arg(
                                Arg::with_name(TARGET_ASSET_ID)
                                    .long(TARGET_ASSET_ID)
                                    .value_name(TARGET_ASSET_ID)
                                    .help("Target Asset's Id as double-quoted string in the following format `asset_name#domain_name`.")
                                    .takes_value(true)
                                    .required(true)
                            )
                    )
                    .subcommand(
                        App::new(REMOVE)
                            .about("Use this command to delete existing Token Pair from the DEX.")
                            .arg(
                                Arg::with_name(BASE_ASSET_ID)
                                    .long(BASE_ASSET_ID)
                                    .value_name(BASE_ASSET_ID)
                                    .help("Base Asset's Id as double-quoted string in the following format `asset_name#domain_name`.")
                                    .takes_value(true)
                                    .required(true)
                            )
                            .arg(
                                Arg::with_name(TARGET_ASSET_ID)
                                    .long(TARGET_ASSET_ID)
                                    .value_name(TARGET_ASSET_ID)
                                    .help("Target Asset's Id as double-quoted string in the following format `asset_name#domain_name`.")
                                    .takes_value(true)
                                    .required(true)
                            )
                    )
                    .subcommand(
                        App::new(GET)
                            .about("Use this command to get information about existing Token Pair from the DEX.")
                            .arg(
                                Arg::with_name(BASE_ASSET_ID)
                                    .long(BASE_ASSET_ID)
                                    .value_name(BASE_ASSET_ID)
                                    .help("Base Asset's Id as double-quoted string in the following format `asset_name#domain_name`.")
                                    .takes_value(true)
                                    .required(true)
                            )
                            .arg(
                                Arg::with_name(TARGET_ASSET_ID)
                                    .long(TARGET_ASSET_ID)
                                    .value_name(TARGET_ASSET_ID)
                                    .help("Target Asset's Id as double-quoted string in the following format `asset_name#domain_name`.")
                                    .takes_value(true)
                                    .required(true)
                            )
                    )
                    .subcommand(
                        App::new(LIST)
                            .about("Use this command to list all active Token Pairs in a DEX.")
                    )
            )
            .subcommand(
                App::new(XYK_POOL_SWAP)
                .about("Use this command to swap tokens via chain of XYK Pools.")
                .arg(
                    Arg::with_name(DOMAIN_NAME)
                        .long(DOMAIN_NAME)
                        .value_name(DOMAIN_NAME)
                        .help("DEX's domain's name as double-quoted string.")
                        .takes_value(true)
                        .required(true)
                )
                .arg(
                    Arg::with_name(PATH)
                        .long(PATH)
                        .value_name(PATH)
                        .help("Path of Asset names via which exchange will happen, written as array of asset names (e.g. [\"ETH\", \"XOR\", \"DOT\"])")
                        .takes_value(true)
                        .required(true)
                )
                .arg(
                    Arg::with_name(OUTPUT_AMOUNT)
                        .long(OUTPUT_AMOUNT)
                        .value_name(OUTPUT_AMOUNT)
                        .help("Output Asset's (last in path) quantity.")
                        .takes_value(true)
                        .required(false)
                )
                .arg(
                    Arg::with_name(INPUT_AMOUNT)
                        .long(INPUT_AMOUNT)
                        .value_name(INPUT_AMOUNT)
                        .help("Input Asset's (first in path) quantity.")
                        .takes_value(true)
                        .required(false)
                )
            )
            .subcommand(
                App::new(XYK_POOL)
                        .about("Use this command to work with LiquditySource of type XYK Pool in active TokenPair.")
                        .arg(
                            Arg::with_name(DOMAIN_NAME)
                                .long(DOMAIN_NAME)
                                .value_name(DOMAIN_NAME)
                                .help("DEX's domain's name as double-quoted string.")
                                .takes_value(true)
                                .required(true)
                        )
                        .arg(
                            Arg::with_name(BASE_ASSET_ID)
                                .long(BASE_ASSET_ID)
                                .value_name(BASE_ASSET_ID)
                                .help("Base Asset's Id as double-quoted string in the following format `asset_name#domain_name`.")
                                .takes_value(true)
                                .required(true)
                        )
                        .arg(
                            Arg::with_name(TARGET_ASSET_ID)
                                .long(TARGET_ASSET_ID)
                                .value_name(TARGET_ASSET_ID)
                                .help("Target Asset's Id as double-quoted string in the following format `asset_name#domain_name`.")
                                .takes_value(true)
                                .required(true)
                        )
                        .subcommand(
                            App::new(CREATE)
                                .about("Use this command to create a new XYK Pool for existing TokenPair.")
                        )
                        .subcommand(
                            App::new(ADD_LIQUIDITY)
                                .about("Use this command to add liquidity to XYK Pool with desired amounts of both tokens in exchange pair.")
                                .arg(
                                    Arg::with_name(BASE_AMOUNT)
                                        .long(BASE_AMOUNT)
                                        .value_name(BASE_AMOUNT)
                                        .help("Desired amount of Base Asset to deposit.")
                                        .takes_value(true)
                                        .required(true)
                                )
                                .arg(
                                    Arg::with_name(TARGET_AMOUNT)
                                        .long(TARGET_AMOUNT)
                                        .value_name(TARGET_AMOUNT)
                                        .help("Desired amount of Target Asset to deposit.")
                                        .takes_value(true)
                                        .required(true)
                                )
                        )
                        .subcommand(
                            App::new(REMOVE_LIQUIDITY)
                            .about("Use this command to remove liquidity from XYK Pool with desired amounts of both tokens in exchange pair.")
                            .arg(
                                Arg::with_name(LIQUIDITY)
                                    .long(LIQUIDITY)
                                    .value_name(LIQUIDITY)
                                    .help("Desired amount of Pool Asset to be burned.")
                                    .takes_value(true)
                                    .required(true))
                        )
            )
    }

    pub fn process(matches: &ArgMatches<'_>) {
        if let Some(ref matches) = matches.subcommand_matches(INITIALIZE) {
            if let Some(domain_name) = matches.value_of(DOMAIN_NAME) {
                println!("Initializing DEX in the domain: {}", domain_name);
                if let Some(base_asset) = matches.value_of(BASE_ASSET_ID) {
                    println!("Initializing DEX with base asset: {}", base_asset);
                    if let Some(owner_account_id) = matches.value_of(OWNER_ACCOUNT_ID) {
                        println!("Initializing DEX with owner account: {}", owner_account_id);
                        initialize_dex(domain_name, owner_account_id, base_asset);
                    }
                }
            }
        }
        if let Some(ref _matches) = matches.subcommand_matches(LIST) {
            println!("Listing all active DEX.");
            list_dex();
        }
        if let Some(ref matches) = matches.subcommand_matches(GET) {
            if let Some(domain_name) = matches.value_of(DOMAIN_NAME) {
                println!("Getting DEX in the domain: {}", domain_name);
                get_dex(domain_name);
            }
        }
        if let Some(ref matches) = matches.subcommand_matches(TOKEN_PAIR) {
            if let Some(domain_name) = matches.value_of(DOMAIN_NAME) {
                if let Some(ref matches) = matches.subcommand_matches(CREATE) {
                    println!("Creating Token Pair in the domain: {}", domain_name);
                    if let Some(base_asset) = matches.value_of(BASE_ASSET_ID) {
                        println!("Creating Token Pair with base asset: {}", base_asset);
                        if let Some(target_asset) = matches.value_of(TARGET_ASSET_ID) {
                            println!("Creating Token Pair with target asset: {}", target_asset);
                            create_token_pair(domain_name, base_asset, target_asset);
                        }
                    }
                }
                if let Some(ref matches) = matches.subcommand_matches(REMOVE) {
                    println!("Removing Token Pair in the domain: {}", domain_name);
                    if let Some(base_asset) = matches.value_of(BASE_ASSET_ID) {
                        println!("Removing Token Pair with base asset: {}", base_asset);
                        if let Some(target_asset) = matches.value_of(TARGET_ASSET_ID) {
                            println!("Removing Token Pair with target asset: {}", target_asset);
                            remove_token_pair(domain_name, base_asset, target_asset);
                        }
                    }
                }
                if let Some(ref _matches) = matches.subcommand_matches(LIST) {
                    println!("Listing active Token Pairs in the domain: {}", domain_name);
                    list_token_pairs(domain_name)
                }
                if let Some(ref matches) = matches.subcommand_matches(GET) {
                    println!("Getting Token Pair in the domain: {}", domain_name);
                    if let Some(base_asset) = matches.value_of(BASE_ASSET_ID) {
                        println!("Getting Token Pair with base asset: {}", base_asset);
                        if let Some(target_asset) = matches.value_of(TARGET_ASSET_ID) {
                            println!("Getting Token Pair with target asset: {}", target_asset);
                            get_token_pair(domain_name, base_asset, target_asset);
                        }
                    }
                }
            }
        }
        if let Some(ref matches) = matches.subcommand_matches(XYK_POOL_SWAP) {
            if let Some(domain_name) = matches.value_of(DOMAIN_NAME) {
                println!("Swapping Tokens in the domain: {}", domain_name);
                if let Some(path) = matches.value_of(PATH) {
                    println!("Swapping Tokens via path: {}", path);
                    match (
                        matches.value_of(INPUT_AMOUNT),
                        matches.value_of(OUTPUT_AMOUNT),
                    ) {
                        (Some(input_amount), None) => {
                            println!("Swapping Exact Tokens For Tokens amount: {}", input_amount);
                            swap_exact_tokens_for_tokens_xyk_pool_cli(
                                domain_name,
                                path,
                                input_amount,
                            );
                        }
                        (None, Some(output_amount)) => {
                            println!("Swapping Tokens for Exact Tokens amount: {}", output_amount);
                            swap_tokens_for_exact_tokens_xyk_pool_cli(
                                domain_name,
                                path,
                                output_amount,
                            );
                        }
                        _ => {
                            panic!("Enter either input or output quantity, not both.");
                        }
                    }
                }
            }
        }
        if let Some(ref matches) = matches.subcommand_matches(XYK_POOL) {
            if let Some(domain_name) = matches.value_of(DOMAIN_NAME) {
                if let Some(base_asset) = matches.value_of(BASE_ASSET_ID) {
                    if let Some(target_asset) = matches.value_of(TARGET_ASSET_ID) {
                        if let Some(ref _matches) = matches.subcommand_matches(CREATE) {
                            println!(
                                "Creating XYK Pool in the domain: {} with base: {} and target: {}",
                                domain_name, base_asset, target_asset
                            );
                            create_xyk_pool_cli(domain_name, base_asset, target_asset);
                        }
                        if let Some(ref matches) = matches.subcommand_matches(ADD_LIQUIDITY) {
                            println!(
                                "Adding liquidity into XYK Pool in the domain: {} with base: {} and target: {}",
                                domain_name, base_asset, target_asset
                            );
                            if let Some(base_amount) = matches.value_of(BASE_AMOUNT) {
                                if let Some(target_amount) = matches.value_of(TARGET_AMOUNT) {
                                    println!(
                                        "Adding liquidity into XYK Pool for Token Pair with Base Asset: {} of quantity: {}",
                                        base_asset, base_amount
                                    );
                                    println!(
                                        "Adding liquidity into XYK Pool for Token Pair with Target Asset: {} of quantity: {}",
                                        target_asset, target_amount
                                    );
                                    add_liquidity_xyk_pool_cli(
                                        domain_name,
                                        base_asset,
                                        target_asset,
                                        base_amount,
                                        target_amount,
                                    );
                                }
                            }
                        }
                        if let Some(ref matches) = matches.subcommand_matches(REMOVE_LIQUIDITY) {
                            println!(
                                "Removing liquidity from XYK Pool in the domain: {} with base: {} and target: {}",
                                domain_name, base_asset, target_asset
                            );
                            if let Some(liquidity) = matches.value_of(LIQUIDITY) {
                                println!(
                                    "Removing liquidity from XYK Pool for Token Pair with Liquidity Amount: {}",
                                    liquidity
                                );
                                remove_liquidity_xyk_pool_cli(
                                    domain_name,
                                    base_asset,
                                    target_asset,
                                    liquidity,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn initialize_dex(domain_name: &str, dex_owner: &str, base_asset: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let owner_account_id = AccountId::from(dex_owner);
        let base_asset_id = AssetDefinitionId::from(base_asset);
        executor::block_on(
            iroha_client.submit(
                isi::Register {
                    object: DEX::new(domain_name, owner_account_id, base_asset_id),
                    destination_id: <Domain as Identifiable>::Id::from(domain_name),
                }
                .into(),
            ),
        )
        .expect("Failed to initialize dex.");
    }

    fn list_dex() {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let query_result = executor::block_on(iroha_client.request(&GetDEXList::build_request()))
            .expect("Failed to list DEX.");
        if let QueryResult::GetDEXList(result) = query_result {
            println!("Get DEX list result: {:#?}", result);
        }
    }

    fn get_dex(domain_name: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let query_result = executor::block_on(iroha_client.request(&GetDEX::build_request(
            <Domain as Identifiable>::Id::from(domain_name),
        )))
        .expect("Failed to get DEX information.");
        if let QueryResult::GetDEX(result) = query_result {
            println!("Get DEX infomation result: {:#?}", result);
        }
    }

    fn create_token_pair(domain_name: &str, base_asset: &str, target_asset: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let dex_id = DEXId::new(domain_name);
        let base_asset_id = AssetDefinitionId::from(base_asset);
        let target_asset_id = AssetDefinitionId::from(target_asset);
        executor::block_on(
            iroha_client.submit(
                isi::Add {
                    object: TokenPair::new(dex_id.clone(), base_asset_id, target_asset_id),
                    destination_id: dex_id,
                }
                .into(),
            ),
        )
        .expect("Failed to create Token Pair.");
    }

    fn remove_token_pair(domain_name: &str, base_asset: &str, target_asset: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let dex_id = DEXId::new(domain_name);
        let base_asset_id = AssetDefinitionId::from(base_asset);
        let target_asset_id = AssetDefinitionId::from(target_asset);
        let token_pair_id = TokenPairId::new(dex_id.clone(), base_asset_id, target_asset_id);
        executor::block_on(
            iroha_client.submit(
                isi::Remove {
                    object: token_pair_id,
                    destination_id: dex_id,
                }
                .into(),
            ),
        )
        .expect("Failed to remove Token Pair.");
    }

    fn list_token_pairs(domain_name: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let query_result = executor::block_on(iroha_client.request(
            &GetTokenPairList::build_request(<Domain as Identifiable>::Id::from(domain_name)),
        ))
        .expect("Failed to list Token Pairs.");
        if let QueryResult::GetTokenPairList(result) = query_result {
            println!("Get TokenPair list result: {:#?}", result);
        }
    }

    fn get_token_pair(domain_name: &str, base_asset: &str, target_asset: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let base_asset_definition_id = AssetDefinitionId::from(base_asset);
        let target_asset_definition_id = AssetDefinitionId::from(target_asset);
        let token_pair_id = TokenPairId::new(
            DEXId::new(domain_name),
            base_asset_definition_id,
            target_asset_definition_id,
        );
        let query_result =
            executor::block_on(iroha_client.request(&GetTokenPair::build_request(token_pair_id)))
                .expect("Failed to get Token Pair information.");
        if let QueryResult::GetTokenPair(result) = query_result {
            println!("Get TokenPair information result: {:#?}", result);
        }
    }

    fn create_xyk_pool_cli(domain_name: &str, base_asset: &str, target_asset: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let base_asset_definition_id = AssetDefinitionId::from(base_asset);
        let target_asset_definition_id = AssetDefinitionId::from(target_asset);
        let token_pair_id = TokenPairId::new(
            DEXId::new(domain_name),
            base_asset_definition_id,
            target_asset_definition_id,
        );
        executor::block_on(iroha_client.submit(xyk_pool::create(token_pair_id)))
            .expect("Failed to create XYK Pool.");
    }

    fn add_liquidity_xyk_pool_cli(
        domain_name: &str,
        base_asset: &str,
        target_asset: &str,
        base_amount: &str,
        target_amount: &str,
    ) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let amount_a_desired = base_amount
            .parse()
            .expect("Failed to parse Asset quantity for Base.");
        let amount_b_desired = target_amount
            .parse()
            .expect("Failed to parse Asset quantity for Target.");
        let amount_a_min = 0u32;
        let amount_b_min = 0u32;
        let base_asset_definition_id = AssetDefinitionId::from(base_asset);
        let target_asset_definition_id = AssetDefinitionId::from(target_asset);
        let token_pair_id = TokenPairId::new(
            DEXId::new(domain_name),
            base_asset_definition_id,
            target_asset_definition_id,
        );
        let liquidity_source_id =
            LiquiditySourceId::new(token_pair_id, LiquiditySourceType::XYKPool);
        executor::block_on(iroha_client.submit(xyk_pool::add_liquidity(
            liquidity_source_id,
            amount_a_desired,
            amount_b_desired,
            amount_a_min,
            amount_b_min,
        )))
        .expect("Failed to add liquidity into XYK Pool.");
    }

    fn remove_liquidity_xyk_pool_cli(
        domain_name: &str,
        base_asset: &str,
        target_asset: &str,
        liquidity_quantity: &str,
    ) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        // prepare command data
        let liquidity_quantity: u32 = liquidity_quantity
            .parse()
            .expect("Failed to parse Asset quantity for Liquidity.");
        let amount_a_min = 0u32;
        let amount_b_min = 0u32;
        let base_asset_definition_id = AssetDefinitionId::from(base_asset);
        let target_asset_definition_id = AssetDefinitionId::from(target_asset);
        let token_pair_id = TokenPairId::new(
            DEXId::new(domain_name),
            base_asset_definition_id,
            target_asset_definition_id,
        );
        let liquidity_source_id =
            LiquiditySourceId::new(token_pair_id, LiquiditySourceType::XYKPool);
        // submit command
        executor::block_on(iroha_client.submit(xyk_pool::remove_liquidity(
            liquidity_source_id,
            liquidity_quantity,
            amount_a_min,
            amount_b_min,
        )))
        .expect("Failed to remove liquidity from XYK Pool.");
    }

    fn swap_exact_tokens_for_tokens_xyk_pool_cli(
        domain_name: &str,
        path: &str,
        input_quantity: &str,
    ) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let input_quantity: u32 = input_quantity
            .parse()
            .expect("Failed to parse Asset quantity for input.");
        let output_quantity_min = 0u32;
        let path = util::string_array_from_str(path)
            .unwrap()
            .iter()
            .map(|asset| AssetDefinitionId::from(asset.as_str()))
            .collect::<Vec<_>>();
        let dex_id = DEXId::new(domain_name);
        executor::block_on(iroha_client.submit(xyk_pool::swap_exact_tokens_for_tokens(
            dex_id,
            path,
            input_quantity,
            output_quantity_min,
        )))
        .expect("Failed to swap exact tokens for tokens via XYK Pools.");
    }

    fn swap_tokens_for_exact_tokens_xyk_pool_cli(
        domain_name: &str,
        path: &str,
        output_quantity: &str,
    ) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let output_quantity: u32 = output_quantity
            .parse()
            .expect("Failed to parse Asset quantity for output.");
        let input_quantity_max = std::u32::MAX;
        let path = util::string_array_from_str(path)
            .unwrap()
            .iter()
            .map(|asset| AssetDefinitionId::from(asset.as_str()))
            .collect::<Vec<_>>();
        let dex_id = DEXId::new(domain_name);
        executor::block_on(iroha_client.submit(xyk_pool::swap_tokens_for_exact_tokens(
            dex_id,
            path,
            output_quantity,
            input_quantity_max,
        )))
        .expect("Failed to swap tokens for exact tokens via XYK Pools.")
    }
}

mod maintenance {
    use super::*;
    use clap::ArgMatches;
    use futures::executor;
    use iroha_client::{client::Client, config::Configuration, prelude::*};

    const HEALTH: &str = "health";
    const METRICS: &str = "metrics";
    const CONNECT: &str = "connect";
    const ENTITY_TYPE: &str = "entity";
    const EVENT_TYPE: &str = "event";

    pub fn build_app<'a, 'b>() -> App<'a, 'b> {
        App::new(MAINTENANCE)
            .about("Use this command to use maintenance functionality.")
            .subcommand(App::new(HEALTH).about("Use this command to check peer's health."))
            .subcommand(App::new(METRICS).about("Use this command to scrape peer's metrics."))
            .subcommand(
                App::new(CONNECT)
                    .about("Use this command to connect to the peer and start consuming events.")
                    .arg(
                        Arg::with_name(ENTITY_TYPE)
                            .long(ENTITY_TYPE)
                            .value_name(ENTITY_TYPE)
                            .help("Type of entity to consume events about.")
                            .takes_value(true)
                            .required(true),
                    )
                    .arg(
                        Arg::with_name(EVENT_TYPE)
                            .long(EVENT_TYPE)
                            .value_name(EVENT_TYPE)
                            .help("Type of event to consume.")
                            .takes_value(true)
                            .required(true),
                    ),
            )
    }

    pub fn process(matches: &ArgMatches<'_>, configuration: &Configuration) {
        if let Some(ref matches) = matches.subcommand_matches(CONNECT) {
            if let Some(entity_type) = matches.value_of(ENTITY_TYPE) {
                println!("Connecting to consume events for: {}", entity_type);
                if let Some(event_type) = matches.value_of(EVENT_TYPE) {
                    println!("Connecting to consume events: {}", event_type);
                    if let Err(err) = connect(entity_type, event_type, configuration) {
                        eprintln!("Failed to connect: {}", err)
                    }
                }
            }
        }
        if matches.subcommand_matches(HEALTH).is_some() {
            println!("Checking peer's health.");
            health(configuration);
        }
        if matches.subcommand_matches(METRICS).is_some() {
            println!("Retrieving peer's metrics.");
            metrics(configuration);
        }
    }

    fn health(configuration: &Configuration) {
        let mut iroha_client = Client::with_maintenance(configuration);
        executor::block_on(async {
            let result = iroha_client
                .health()
                .await
                .expect("Failed to execute request.");
            println!("Health is {:?}", result);
        });
    }

    fn metrics(configuration: &Configuration) {
        let mut iroha_client = Client::with_maintenance(configuration);
        executor::block_on(async {
            let result = iroha_client
                .scrape_metrics()
                .await
                .expect("Failed to execute request.");
            println!("{:?}", result);
        });
    }

    fn connect(
        entity_type: &str,
        event_type: &str,
        configuration: &Configuration,
    ) -> Result<(), String> {
        let mut iroha_client = Client::with_maintenance(configuration);
        let event_type: OccurrenceType = event_type.parse()?;
        let entity_type: EntityType = entity_type.parse()?;
        executor::block_on(async {
            let stream = iroha_client
                .subscribe_to_changes(event_type, entity_type)
                .await
                .expect("Failed to execute request.");
            println!("Successfully connected. Listening for changes.");
            for change in stream {
                println!("Change received {:?}", change);
            }
        });
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use async_std::task;
        use iroha::{config::Configuration, isi, prelude::*};
        use iroha_client::{client::Client, config::Configuration as ClientConfiguration};
        use std::{thread, time::Duration};
        use tempfile::TempDir;

        const CONFIGURATION_PATH: &str = "tests/test_config.json";

        #[test]
        fn cli_check_health_should_work() {
            let mut configuration = Configuration::from_path(CONFIGURATION_PATH)
                .expect("Failed to load configuration.");
            let client_configuration =
                ClientConfiguration::from_iroha_configuration(&configuration);
            task::spawn(async move {
                let temp_dir = TempDir::new().expect("Failed to create TempDir.");
                configuration
                    .kura_configuration
                    .kura_block_store_path(temp_dir.path());
                let iroha = Iroha::new(configuration);
                iroha.start().await.expect("Failed to start Iroha.");
                //Prevents temp_dir from clean up untill the end of the tests.
                #[allow(clippy::empty_loop)]
                loop {}
            });
            thread::sleep(Duration::from_millis(300));
            super::health(&client_configuration);
        }

        #[test]
        fn cli_scrape_metrics_should_work() {
            let mut configuration = Configuration::from_path(CONFIGURATION_PATH)
                .expect("Failed to load configuration.");
            let client_configuration =
                ClientConfiguration::from_iroha_configuration(&configuration);
            task::spawn(async move {
                let temp_dir = TempDir::new().expect("Failed to create TempDir.");
                configuration
                    .kura_configuration
                    .kura_block_store_path(temp_dir.path());
                let iroha = Iroha::new(configuration.clone());
                iroha.start().await.expect("Failed to start Iroha.");
                //Prevents temp_dir from clean up untill the end of the tests.
                #[allow(clippy::empty_loop)]
                loop {}
            });
            thread::sleep(Duration::from_millis(300));
            super::metrics(&client_configuration);
        }

        #[test]
        fn cli_connect_to_consume_block_changes_should_work() {
            let mut configuration = Configuration::from_path(CONFIGURATION_PATH)
                .expect("Failed to load configuration.");
            let client_configuration =
                ClientConfiguration::from_iroha_configuration(&configuration);
            task::spawn(async move {
                let temp_dir = TempDir::new().expect("Failed to create TempDir.");
                configuration
                    .kura_configuration
                    .kura_block_store_path(temp_dir.path());
                let iroha = Iroha::new(configuration.clone());
                iroha.start().await.expect("Failed to start Iroha.");
                //Prevents temp_dir from clean up untill the end of the tests.
                #[allow(clippy::empty_loop)]
                loop {}
            });
            thread::sleep(Duration::from_millis(600));
            let client_configuration_clone = client_configuration.clone();
            thread::spawn(move || super::connect("all", "all", &client_configuration_clone));
            let domain_name = "global";
            let asset_definition_id = AssetDefinitionId::new("xor", domain_name);
            let create_asset = isi::Register {
                object: AssetDefinition::new(asset_definition_id),
                destination_id: domain_name.to_string(),
            };
            let mut iroha_client = Client::new(&client_configuration);
            thread::sleep(Duration::from_millis(300));
            task::block_on(iroha_client.submit(create_asset.into()))
                .expect("Failed to prepare state.");
        }
    }
}
