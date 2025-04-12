use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(crate = "rocket::serde")]
pub struct Post {
    // FIXME: This is because Masto sends an array but we don't care
    #[serde(rename = "@context")]
    #[serde(skip_deserializing)]
    pub context: String,
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(rename = "published")]
    pub ts: String,
    pub content: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
}
