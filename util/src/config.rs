//! This module contain application configuration related functionality.
//!
//! All the application configurations should be set in corresponding
//! TOML file in `config` directory.
use config::{Config, ConfigError, Environment, File};

use crate::str_util::{mask_creds, mask_url_passwd};
use serde::Deserialize;
use std::net::Ipv4Addr;
use std::{
    fmt,
    path::{Path, PathBuf},
};

const DEFAULT_CONFIG_FILE_PREFIX: &str = "config";
const DEFAULT_CONFIG_FILE_NAME: &str = "default.toml";

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum EnvProfile {
    Prod,
    Local,
    Dev,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpServer {
    pub port: u16,
    pub host: Ipv4Addr,
    pub log_level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JsonRpc {
    pub port: u16,
    pub host: Ipv4Addr,
    pub log_level: String,
}

#[derive(Deserialize, Clone)]
pub struct ObjStorage {
    pub region: Option<String>,
    pub endpoint: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
    pub bucket_for_json_metadata: String,
}

impl fmt::Debug for ObjStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjStorage")
            .field("region", &self.region)
            .field("endpoint", &self.endpoint)
            .field(
                "access_key_id",
                &self.access_key_id.as_ref().map(|s| mask_creds(s)),
            )
            .field(
                "secret_access_key",
                &self.secret_access_key.as_ref().map(|s| mask_creds(s)),
            )
            .field(
                "session_token",
                &self.session_token.as_ref().map(|s| mask_creds(s)),
            )
            .field("bucket_for_json_metadata", &self.bucket_for_json_metadata)
            .finish()
    }
}

#[derive(Deserialize, Clone)]
pub struct DatabaseCfg {
    pub connection_url: String,
    pub min_connections: u32,
    pub max_connections: u32,
}

impl fmt::Debug for DatabaseCfg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseCfg")
            .field("connection_url", &mask_url_passwd(&self.connection_url))
            .field("min_connections", &self.min_connections)
            .field("max_connections", &self.max_connections)
            .finish()
    }
}

#[allow(unused)]
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub http_server: HttpServer,
    pub json_rpc_server: JsonRpc,
    pub obj_storage: ObjStorage,
    pub database: DatabaseCfg,
    pub env: EnvProfile,
}

impl Settings {
    pub fn for_env(env_name: &str) -> Result<Self, ConfigError> {
        Settings::load(Some(env_name), None)
    }

    /// This method should be used for production.
    /// It loads application configuration based on the environment variables.
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Result<Self, ConfigError> {
        Settings::load(None, None)
    }

    pub fn is_production_profile(&self) -> bool {
        self.env.eq(&EnvProfile::Prod)
    }

    pub fn is_not_production_profile(&self) -> bool {
        !self.is_production_profile()
    }

    fn load(env_name: Option<&str>, config_path: Option<&str>) -> Result<Self, ConfigError> {
        let configs_path = config_path.map(|s| s.to_string()).unwrap_or(
            std::env::var("RUN_CONFIG_DIR")
                .unwrap_or_else(|_| DEFAULT_CONFIG_FILE_PREFIX.to_string()),
        );

        let env = env_name
            .map(|s| s.to_string())
            .unwrap_or(std::env::var("RUN_ENV").unwrap_or_else(|_| "local".into()));
        println!("Using profile: {}", &env);

        let raw_config = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::from(
                default_config_file_path(&configs_path).as_path(),
            ))
            // Add in the current environment file, Default to 'development' env
            // Note that this file is _optional_
            .add_source(File::with_name(&format!("{}/{}", configs_path, env)).required(false))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_SERVER__PORT=8081 ./target/app` would set the `port` key
            .add_source(Environment::with_prefix("app").separator("__"))
            .set_override("env", env)?
            .build()?;

        raw_config.try_deserialize()
    }
}

fn default_config_file_path(base_path: &str) -> PathBuf {
    // Check if the base path is a full path
    let full_path = Path::new(base_path);
    if full_path.exists() {
        return full_path.to_owned();
    }

    // it's OK to unwrap(), since it's the initialization phase,
    // and it's better to fail fast in case of a problem.
    let current_dir = std::env::current_dir().unwrap();

    let mut config_dir = current_dir.join(base_path);
    if !config_dir.exists() {
        config_dir = current_dir.parent().unwrap().join(base_path);
    }

    config_dir.join(DEFAULT_CONFIG_FILE_NAME)
}
