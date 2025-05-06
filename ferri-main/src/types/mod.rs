use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Debug;
use thiserror::Error;
use uuid::Uuid;

pub mod convert;
pub mod get;
pub mod make;

pub mod db;
pub mod ap;
pub mod api;

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("an unknown error occured when creating: {0}")]
    CreationError(String),
    #[error("an unknown error occured when fetching: {0}")]
    FetchError(String),
}

pub const AS_CONTEXT_RAW: &str = "https://www.w3.org/ns/activitystreams";
pub fn as_context() -> ObjectContext {
    ObjectContext::Str(AS_CONTEXT_RAW.to_string())
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum ObjectContext {
    Str(String),
    Vec(Vec<serde_json::Value>),
}

impl Default for ObjectContext {
    fn default() -> Self {
        ObjectContext::Str(String::new())
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct ObjectUri(pub String);

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct ObjectUuid(pub String);

impl Default for ObjectUuid {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectUuid {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Object {
    #[serde(rename = "@context")]
    #[serde(default)]
    pub context: ObjectContext,
    pub id: ObjectUri,
}
