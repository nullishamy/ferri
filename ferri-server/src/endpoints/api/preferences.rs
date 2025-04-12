use rocket::{
    get,
    serde::{Deserialize, Serialize, json::Json},
};

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Preferences {
    #[serde(rename = "posting:default:visibility")]
    pub posting_default_visibility: String,
    #[serde(rename = "posting:default:sensitive")]
    pub posting_default_sensitive: bool,
    #[serde(rename = "posting:default:language")]
    pub posting_default_language: Option<String>,
    #[serde(rename = "reading:expand:media")]
    pub reading_expand_media: String,
    #[serde(rename = "reading:expand:spoilers")]
    pub reading_expand_spoilers: bool,
}

#[get("/preferences")]
pub async fn preferences() -> Json<Preferences> {
    Json(Preferences {
        posting_default_visibility: "public".to_string(),
        posting_default_sensitive: false,
        posting_default_language: None,
        reading_expand_media: "default".to_string(),
        reading_expand_spoilers: false,
    })
}
