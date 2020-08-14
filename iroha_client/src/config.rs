use iroha::{config::Configuration as IrohaConfiguration, crypto::PrivateKey, crypto::PublicKey};
use iroha_derive::*;
use serde::Deserialize;
use std::{env, fmt::Debug, fs::File, io::BufReader, path::Path};

const TORII_URL: &str = "TORII_URL";
const TORII_CONNECT_URL: &str = "TORII_CONNECT_URL";
const IROHA_PUBLIC_KEY: &str = "IROHA_PUBLIC_KEY";
const DEFAULT_TORII_URL: &str = "127.0.0.1:1337";
const DEFAULT_TORII_CONNECT_URL: &str = "127.0.0.1:8888";
const DEFUALT_ACCOUNT_NAME: &str = "root";
const DEFAULT_DOMAIN_NAME: &str = "global";

/// `Configuration` provides an ability to define client parameters such as `TORII_URL`.
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub struct Configuration {
    /// Public key of this client.
    pub public_key: PublicKey,
    /// Private key of this client.
    pub private_key: PrivateKey,
    /// Torii URL.
    #[serde(default = "default_torii_url")]
    pub torii_url: String,
    /// Torii connection URL.
    #[serde(default = "default_torii_connect_url")]
    pub torii_connect_url: String,
    /// Account name of client.
    #[serde(default = "default_account_name")]
    pub account_name: String,
    /// Domain of client's account.
    #[serde(default = "default_domain_name")]
    pub domain_name: String,
}

impl Configuration {
    /// This method will build `Configuration` from a json *pretty* formatted file (without `:` in
    /// key names).
    /// # Panics
    /// This method will panic if configuration file presented, but has incorrect scheme or format.
    /// # Errors
    /// This method will return error if system will fail to find a file or read it's content.
    #[log]
    pub fn from_path<P: AsRef<Path> + Debug>(path: P) -> Result<Configuration, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open a file: {}", e))?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to deserialize json from reader: {}", e))?)
    }

    /// This method will build `Configuration` from existing `IrohaConfiguration`.
    #[log]
    pub fn from_iroha_configuration(configuration: &IrohaConfiguration) -> Self {
        Configuration {
            torii_url: configuration.torii_configuration.torii_url.clone(),
            public_key: configuration.public_key.clone(),
            private_key: configuration.private_key.clone(),
            torii_connect_url: default_torii_connect_url(),
            account_name: default_account_name(),
            domain_name: default_domain_name(),
        }
    }

    /// Load environment variables and replace predefined parameters with these variables
    /// values.
    #[log]
    pub fn load_environment(&mut self) -> Result<(), String> {
        if let Ok(torii_url) = env::var(TORII_URL) {
            self.torii_url = torii_url;
        }
        if let Ok(torii_connect_url) = env::var(TORII_CONNECT_URL) {
            self.torii_connect_url = torii_connect_url;
        }
        if let Ok(public_key) = env::var(IROHA_PUBLIC_KEY) {
            self.public_key = serde_json::from_str(&public_key)
                .map_err(|e| format!("Failed to parse Public Key: {}", e))?;
        }
        Ok(())
    }
}

fn default_torii_url() -> String {
    DEFAULT_TORII_URL.to_string()
}

fn default_torii_connect_url() -> String {
    DEFAULT_TORII_CONNECT_URL.to_string()
}

fn default_account_name() -> String {
    DEFUALT_ACCOUNT_NAME.to_string()
}

fn default_domain_name() -> String {
    DEFAULT_DOMAIN_NAME.to_string()
}
