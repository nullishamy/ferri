pub mod config;
pub mod types;
pub mod federation;

use chrono::{DateTime, Utc};
use rand::{Rng, distributions::Alphanumeric};

pub fn gen_token(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn now() -> DateTime<Utc> {
    Utc::now()
}

pub fn now_str() -> String {
    now().to_rfc3339()
}
