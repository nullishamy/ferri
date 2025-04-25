use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod http;

mod activity;
pub use activity::*;

mod user;
pub use user::*;

mod post;
pub use post::*;

pub const AS_CONTEXT: &str = "https://www.w3.org/ns/activitystreams";

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn new_ts() -> String {
    now().to_rfc3339()
}

pub fn now() -> DateTime<Utc> {
    Utc::now()
}
