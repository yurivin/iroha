use clap::{App, Arg};

const CONFIG: &str = "config";
const DOMAIN: &str = "domain";
const ACCOUNT: &str = "account";
const ASSET: &str = "asset";
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
            dex::build_app(),
        )
        .get_matches();
    if let Some(configuration_path) = matches.value_of(CONFIG) {
        println!("Value for config: {}", configuration_path);
    }
    if let Some(ref matches) = matches.subcommand_matches(DOMAIN) {
        domain::process(matches);
    }
    if let Some(ref matches) = matches.subcommand_matches(ACCOUNT) {
        account::process(matches);
    }
    if let Some(ref matches) = matches.subcommand_matches(ASSET) {
        asset::process(matches);
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

    pub fn process(matches: &ArgMatches<'_>) {
        if let Some(ref matches) = matches.subcommand_matches(ADD) {
            if let Some(domain_name) = matches.value_of(DOMAIN_NAME) {
                println!("Adding a new Domain with a name: {}", domain_name);
                create_domain(domain_name);
            }
        }
    }

    fn create_domain(domain_name: &str) {
        let configuration =
            &Configuration::from_path("config.json").expect("Failed to load configuration.");
        let mut iroha_client = Client::new(&configuration);
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
    use iroha::{isi, prelude::*};
    use iroha_client::{client::util::*, client::Client, config::Configuration};

    const REGISTER: &str = "register";
    const ACCOUNT_NAME: &str = "name";
    const ACCOUNT_DOMAIN_NAME: &str = "domain";
    const ACCOUNT_KEY: &str = "key";

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
    }

    pub fn process(matches: &ArgMatches<'_>) {
        if let Some(ref matches) = matches.subcommand_matches(REGISTER) {
            if let Some(account_name) = matches.value_of(ACCOUNT_NAME) {
                println!("Creating account with a name: {}", account_name);
                if let Some(domain_name) = matches.value_of(ACCOUNT_DOMAIN_NAME) {
                    println!("Creating account with a domain's name: {}", domain_name);
                    if let Some(public_key) = matches.value_of(ACCOUNT_KEY) {
                        println!("Creating account with a public key: {}", public_key);
                        create_account(account_name, domain_name, public_key);
                    }
                }
            }
        }
    }

    fn create_account(account_name: &str, domain_name: &str, public_key: &str) {
        let public_key = public_key_from_str(public_key).unwrap();
        let create_account = isi::Register {
            object: Account::with_signatory(account_name, domain_name, public_key),
            destination_id: String::from(domain_name),
        };
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        executor::block_on(iroha_client.submit(create_account.into()))
            .expect("Failed to create account.");
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
                    .arg(Arg::with_name(ASSET_ACCOUNT_ID).long(ASSET_ACCOUNT_ID).value_name(ASSET_ACCOUNT_ID).help("Account's id as double-quoted string in the following format `account_name@domain_name`.").takes_value(true).required(true))
                    .arg(Arg::with_name(ASSET_ID).long(ASSET_ID).value_name(ASSET_ID).help("Asset's id as double-quoted string in the following format `asset_name#domain_name`.").takes_value(true).required(true))
                    .arg(Arg::with_name(QUANTITY).long(QUANTITY).value_name(QUANTITY).help("Asset's quantity as a number.").takes_value(true).required(true))
                )
                .subcommand(
                    App::new(GET)
                    .about("Use this command to get Asset information from Iroha Account.")
                        .arg(Arg::with_name(ASSET_ACCOUNT_ID).long(ASSET_ACCOUNT_ID).value_name(ASSET_ACCOUNT_ID).help("Account's id as double-quoted string in the following format `account_name@domain_name`.").takes_value(true).required(true))
                        .arg(Arg::with_name(ASSET_ID).long(ASSET_ID).value_name(ASSET_ID).help("Asset's id as double-quoted string in the following format `asset_name#domain_name`.").takes_value(true).required(true))

            )
    }

    pub fn process(matches: &ArgMatches<'_>) {
        if let Some(ref matches) = matches.subcommand_matches(REGISTER) {
            if let Some(asset_name) = matches.value_of(ASSET_NAME) {
                println!("Registering asset defintion with a name: {}", asset_name);
                if let Some(domain_name) = matches.value_of(ASSET_DOMAIN_NAME) {
                    println!(
                        "Registering asset definition with a domain's name: {}",
                        domain_name
                    );
                    register_asset_definition(asset_name, domain_name);
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
                        mint_asset(asset_id, account_id, amount);
                    }
                }
            }
        }
        if let Some(ref matches) = matches.subcommand_matches(GET) {
            if let Some(asset_id) = matches.value_of(ASSET_ID) {
                println!("Getting asset with an identification: {}", asset_id);
                if let Some(account_id) = matches.value_of(ASSET_ACCOUNT_ID) {
                    println!("Getting account with an identification: {}", account_id);
                    get_asset(asset_id, account_id);
                }
            }
        }
    }

    fn register_asset_definition(asset_name: &str, domain_name: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
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

    fn mint_asset(asset_definition_id: &str, account_id: &str, quantity: &str) {
        let quantity: u32 = quantity.parse().expect("Failed to parse Asset quantity.");
        let mint_asset = isi::Mint {
            object: quantity,
            destination_id: AssetId {
                definition_id: AssetDefinitionId::from(asset_definition_id),
                account_id: AccountId::from(account_id),
            },
        };
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        executor::block_on(iroha_client.submit(mint_asset.into()))
            .expect("Failed to create account.");
    }

    fn get_asset(_asset_id: &str, account_id: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let query_result = executor::block_on(iroha_client.request(
            &client::assets::by_account_id(<Account as Identifiable>::Id::from(account_id)),
        ))
        .expect("Failed to get asset.");
        if let QueryResult::GetAccountAssets(result) = query_result {
            println!("Get Asset result: {:?}", result);
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
        client::{self, Client},
        config::Configuration,
    };

    const INITIALIZE: &str = "initialize";
    const TOKEN_PAIR: &str = "token_pair";
    const CREATE: &str = "create";
    const REMOVE: &str = "remove";
    const LIST: &str = "list";
    const GET: &str = "get";
    const ACCOUNT_NAME: &str = "account";
    const OWNER_ACCOUNT_ID: &str = "owner_account_id";
    const DOMAIN_NAME: &str = "domain";
    const BASE: &str = "base";
    const TARGET: &str = "target";
    const XYK_POOL: &str = "xyk_pool";
    const BASE_AMOUNT: &str = "base_amount";
    const TARGET_AMOUNT: &str = "target_amount";
    const LIQUIDITY: &str = "liquidity";
    const ADD_LIQUIDITY: &str = "add_liquidity";
    const REMOVE_LIQUIDITY: &str = "remove_liquidity";
    const ACTIVATE_ACCOUNT: &str = "activate_account";

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
            )
            .subcommand(
                App::new(LIST)
                    .about("Use this command to list all active DEX in Iroha Peer.")
            )
            .subcommand(
                App::new(GET)
                    .about("Use this command to get particular DEX information.")
                    .arg(Arg::with_name(DOMAIN_NAME).long(DOMAIN_NAME).value_name(DOMAIN_NAME).help("DEX's domain's name as double-quoted string.").takes_value(true).required(true))
            )
            .subcommand(
                App::new(TOKEN_PAIR)
                    .about("Use this command to work with TokenPair Entities in active DEX.")
                    .arg(Arg::with_name(DOMAIN_NAME).long(DOMAIN_NAME).value_name(DOMAIN_NAME).help("DEX's domain's name as double-quoted string.").takes_value(true).required(true))
                    .subcommand(
                        App::new(CREATE)
                            .about("Use this command to create a new Token Pair for existing Assets.")
                            .arg(Arg::with_name(BASE).long(BASE).value_name(BASE).help("Base Asset's name without domain indication.").takes_value(true).required(true))
                            .arg(Arg::with_name(TARGET).long(TARGET).value_name(TARGET).help("Target Asset's name without domain indication.").takes_value(true).required(true))
                    )
                    .subcommand(
                        App::new(REMOVE)
                            .about("Use this command to delete existing Token Pair from the DEX.")
                            .arg(Arg::with_name(BASE).long(BASE).value_name(BASE).help("Base Asset's name without domain indication.").takes_value(true).required(true))
                            .arg(Arg::with_name(TARGET).long(TARGET).value_name(TARGET).help("Target Asset's name without domain indication.").takes_value(true).required(true))
                    )
                    .subcommand(
                        App::new(GET)
                            .about("Use this command to get information about existing Token Pair from the DEX.")
                            .arg(Arg::with_name(BASE).long(BASE).value_name(BASE).help("Base Asset's name without domain indication.").takes_value(true).required(true))
                            .arg(Arg::with_name(TARGET).long(TARGET).value_name(TARGET).help("Target Asset's name without domain indication.").takes_value(true).required(true))
                    )
                    .subcommand(
                        App::new(LIST)
                            .about("Use this command to list all active Token Pairs in a DEX.")
                    )
            )
            .subcommand(
                App::new(XYK_POOL)
                        .about("Use this command to work with LiquditySource of type XYK Pool in active TokenPair.")
                        .arg(Arg::with_name(DOMAIN_NAME).long(DOMAIN_NAME).value_name(DOMAIN_NAME).help("DEX's domain's name as double-quoted string.").takes_value(true).required(true))
                        .arg(Arg::with_name(BASE).long(BASE).value_name(BASE).help("Base Asset's name without domain indication.").takes_value(true).required(true))
                        .arg(Arg::with_name(TARGET).long(TARGET).value_name(TARGET).help("Target Asset's name without domain indication.").takes_value(true).required(true))
                        .subcommand(
                            App::new(CREATE)
                            .about("Use this command to create a new XYK Pool for existing TokenPair.")
                        )
                        .subcommand(
                            App::new(ADD_LIQUIDITY)
                            .about("Use this command to add liquidity to XYK Pool with desired amounts of both tokens in exchange pair.")
                            .arg(Arg::with_name(BASE_AMOUNT).long(BASE_AMOUNT).value_name(BASE_AMOUNT).help("Desired amount of Base Asset to deposit.").takes_value(true).required(true))
                            .arg(Arg::with_name(TARGET_AMOUNT).long(TARGET_AMOUNT).value_name(TARGET_AMOUNT).help("Desired amount of Target Asset to deposit.").takes_value(true).required(true))
                        )
                        .subcommand(
                            App::new(REMOVE_LIQUIDITY)
                            .about("Use this command to remove liquidity from XYK Pool with desired amounts of both tokens in exchange pair.")
                            .arg(Arg::with_name(LIQUIDITY).long(LIQUIDITY).value_name(LIQUIDITY).help("Desired amount of Liquidity Asset to be burned.").takes_value(true).required(true))
                        )
                        .subcommand(
                            App::new(ACTIVATE_ACCOUNT)
                            .about("Use this command to activate account for trading on this pool.")
                            .arg(Arg::with_name(ACCOUNT_NAME).long(ACCOUNT_NAME).value_name(ACCOUNT_NAME).help("Account name without domain indication.").takes_value(true).required(true))
                        )
            )
    }

    pub fn process(matches: &ArgMatches<'_>) {
        if let Some(ref matches) = matches.subcommand_matches(INITIALIZE) {
            if let Some(domain_name) = matches.value_of(DOMAIN_NAME) {
                println!("Initializing DEX in the domain: {}", domain_name);
                if let Some(owner_account_id) = matches.value_of(OWNER_ACCOUNT_ID) {
                    println!("Initializing DEX with owner account: {}", owner_account_id);
                    initialize_dex(domain_name, owner_account_id);
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
                    if let Some(base_asset) = matches.value_of(BASE) {
                        println!(
                            "Creating Token Pair with base asset: {}#{}",
                            base_asset, domain_name
                        );
                        if let Some(target_asset) = matches.value_of(TARGET) {
                            println!(
                                "Creating Token Pair with target asset: {}#{}",
                                target_asset, domain_name
                            );
                            create_token_pair(domain_name, base_asset, target_asset);
                        }
                    }
                }
                if let Some(ref matches) = matches.subcommand_matches(REMOVE) {
                    println!("Removing Token Pair in the domain: {}", domain_name);
                    if let Some(base_asset) = matches.value_of(BASE) {
                        println!("Removing Token Pair with base asset: {}", base_asset);
                        if let Some(target_asset) = matches.value_of(TARGET) {
                            println!("Removing Token Pair with target asset: {}", target_asset);
                            remove_token_pair(domain_name, base_asset, target_asset);
                        }
                    }
                }
                if let Some(ref matches) = matches.subcommand_matches(LIST) {
                    println!("Listing active Token Pairs in the domain: {}", domain_name);
                    list_token_pairs(domain_name)
                }
                if let Some(ref matches) = matches.subcommand_matches(GET) {
                    println!("Getting Token Pair in the domain: {}", domain_name);
                    if let Some(base_asset) = matches.value_of(BASE) {
                        println!("Getting Token Pair with base asset: {}", base_asset);
                        if let Some(target_asset) = matches.value_of(TARGET) {
                            println!("Getting Token Pair with target asset: {}", target_asset);
                            get_token_pair(domain_name, base_asset, target_asset);
                        }
                    }
                }
            }
        }
        if let Some(ref matches) = matches.subcommand_matches(XYK_POOL) {
            if let Some(domain_name) = matches.value_of(DOMAIN_NAME) {
                if let Some(base_asset) = matches.value_of(BASE) {
                    if let Some(target_asset) = matches.value_of(TARGET) {
                        if let Some(ref matches) = matches.subcommand_matches(CREATE) {
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
                                        base_asset, target_amount
                                    );
                                    add_liquidity(
                                        domain_name,
                                        base_asset,
                                        target_asset,
                                        base_amount,
                                        target_amount,
                                    );
                                }
                            }
                        }
                        if let Some(ref matches) = matches.subcommand_matches(ACTIVATE_ACCOUNT) {
                            if let Some(account_name) = matches.value_of(ACCOUNT_NAME) {
                                println!("Activating XYK Pool trading account: {}", account_name);
                                activate_account_xyk_pool(
                                    domain_name,
                                    account_name,
                                    base_asset,
                                    target_asset,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn initialize_dex(domain_name: &str, dex_owner: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let owner_account_id = AccountId::from(dex_owner);
        executor::block_on(
            iroha_client.submit(
                isi::Register {
                    object: DEX::new(domain_name, owner_account_id),
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
            println!("Get DEX list result: {:?}", result);
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
            println!("Get DEX infomation result: {:?}", result);
        }
    }

    fn create_token_pair(domain_name: &str, base_asset_name: &str, target_asset_name: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let dex_id = DEXId::new(domain_name);
        let base_asset_id = AssetDefinitionId::new(base_asset_name, domain_name);
        let target_asset_id = AssetDefinitionId::new(target_asset_name, domain_name);
        executor::block_on(
            iroha_client.submit(
                isi::Add {
                    object: TokenPair::new(dex_id.clone(), base_asset_id, target_asset_id, 0, 0),
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
        let base_asset_id = AssetDefinitionId::new(base_asset, domain_name);
        let target_asset_id = AssetDefinitionId::new(target_asset, domain_name);
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
            println!("Get TokenPair list result: {:?}", result);
        }
    }

    fn get_token_pair(domain_name: &str, base_asset: &str, target_asset: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let base_asset_definition_id = AssetDefinitionId::new(base_asset, domain_name);
        let target_asset_definition_id = AssetDefinitionId::new(target_asset, domain_name);
        let token_pair_id = TokenPairId::new(
            DEXId::new(domain_name),
            base_asset_definition_id,
            target_asset_definition_id,
        );
        let query_result =
            executor::block_on(iroha_client.request(&GetTokenPair::build_request(token_pair_id)))
                .expect("Failed to get Token Pair information.");
        if let QueryResult::GetTokenPair(result) = query_result {
            println!("Get TokenPair information result: {:?}", result);
        }
    }

    fn create_xyk_pool_cli(domain_name: &str, base_asset: &str, target_asset: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let base_asset_definition_id = AssetDefinitionId::new(base_asset, domain_name);
        let target_asset_definition_id = AssetDefinitionId::new(target_asset, domain_name);
        let token_pair_id = TokenPairId::new(
            DEXId::new(domain_name),
            base_asset_definition_id,
            target_asset_definition_id,
        );
        executor::block_on(iroha_client.submit(create_xyk_pool(token_pair_id)))
            .expect("Failed to create XYK Pool.");
    }

    fn activate_account_xyk_pool(
        domain_name: &str,
        account_name: &str,
        base_asset: &str,
        target_asset: &str,
    ) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let account_id = AccountId::new(account_name, domain_name);
        let base_asset_definition_id = AssetDefinitionId::new(base_asset, domain_name);
        let target_asset_definition_id = AssetDefinitionId::new(target_asset, domain_name);
        let token_pair_id = TokenPairId::new(
            DEXId::new(domain_name),
            base_asset_definition_id,
            target_asset_definition_id,
        );
        let liquidity_source_id =
            LiquiditySourceId::new(token_pair_id, LiquiditySourceType::XYKPool);
        let instruction = Instruction::DEX(DEXInstruction::ActivateXYKPoolTraderAccount(
            liquidity_source_id,
            account_id,
        ));
        executor::block_on(iroha_client.submit(instruction))
            .expect("Failed to activate account for XYK Pool use.")
    }

    fn add_liquidity(
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
        let base_asset_definition_id = AssetDefinitionId::new(base_asset, domain_name);
        let target_asset_definition_id = AssetDefinitionId::new(target_asset, domain_name);
        let token_pair_id = TokenPairId::new(
            DEXId::new(domain_name),
            base_asset_definition_id,
            target_asset_definition_id,
        );
        let liquidity_source_id =
            LiquiditySourceId::new(token_pair_id, LiquiditySourceType::XYKPool);
        executor::block_on(iroha_client.submit(xyk_pool_add_liquidity(
            liquidity_source_id,
            amount_a_desired,
            amount_b_desired,
            amount_a_min,
            amount_b_min,
        )))
        .expect("Failed to add liquidity into XYK Pool.");
    }

    fn remove_liquidity(domain_name: &str, base_asset: &str, target_asset: &str, liquidity: &str) {
        let mut iroha_client = Client::new(
            &Configuration::from_path("config.json").expect("Failed to load configuration."),
        );
        let liquidity_quantity: u32 = liquidity
            .parse()
            .expect("Failed to parse Asset quantity for Liquidity.");
        let amount_a_min = 0u32;
        let amount_b_min = 0u32;
        unimplemented!()
    }
}
