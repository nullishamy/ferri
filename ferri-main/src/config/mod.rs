use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub host: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub server: ServerConfig,
}
