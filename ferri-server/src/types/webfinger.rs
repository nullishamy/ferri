use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Link {
    pub rel: String,
    #[serde(rename = "type")]
    pub ty: Option<String>,
    pub href: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct WebfingerResponse {
    pub subject: String,
    pub aliases: Vec<String>,
    pub links: Vec<Link>,
}