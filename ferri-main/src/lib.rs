pub mod ap;
pub mod config;
pub mod types;
pub mod federation;

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
