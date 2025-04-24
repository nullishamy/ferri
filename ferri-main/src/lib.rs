pub mod ap;
pub mod config;
use rand::{Rng, distributions::Alphanumeric};

pub fn gen_token(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
