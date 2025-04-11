use rocket::{serde::{Deserialize, Serialize}, FromForm};

#[derive(Serialize, Deserialize, Debug, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct App {
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub scopes: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct CredentialApplication {
    pub name: String,
    pub scopes: String,
    pub redirect_uris: Vec<String>,
    pub client_id: String,
    pub client_secret: String,
}