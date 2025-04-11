pub mod activity;
pub mod content;
pub mod instance;
pub mod oauth;
pub mod webfinger;

use rocket::serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(crate = "rocket::serde")]
pub struct UserKey {
    pub id: String,
    pub owner: String,
    #[serde(rename = "publicKeyPem")]
    pub public_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(crate = "rocket::serde")]
pub struct Person {
    // FIXME: This is because Masto sends an array but we don't care
    #[serde(rename = "@context")]
    #[serde(skip_deserializing)]
    pub context: String,

    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub following: String,
    pub followers: String,
    pub inbox: String,
    pub outbox: String,
    pub preferred_username: String,
    pub name: String,
    pub summary: String,
    pub public_key: Option<UserKey>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(crate = "rocket::serde")]
pub struct Object {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub object: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(crate = "rocket::serde")]
pub struct OrderedCollection {
    pub summary: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub total_items: u64,
    pub ordered_items: Vec<String>,
}
