use rocket::serde::{Deserialize, Serialize};

use crate::types::content::Post;

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct MinimalActivity {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct DeleteActivity {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,

    pub object: String,
    pub actor: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct CreateActivity {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,

    pub object: Post,
    pub actor: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,

    #[serde(rename = "published")]
    pub ts: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct FollowActivity {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,

    pub object: String,
    pub actor: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct AcceptActivity {
    #[serde(rename = "type")]
    pub ty: String,

    pub object: String,
    pub actor: String,
}