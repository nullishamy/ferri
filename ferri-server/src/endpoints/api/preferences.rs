use main::types::api;
use rocket::{
    get,
    serde::json::Json,
};

#[get("/preferences")]
pub async fn preferences() -> Json<api::Preferences> {
    Json(api::Preferences {
        posting_default_visibility: "public".to_string(),
        posting_default_sensitive: false,
        posting_default_language: None,
        reading_expand_media: "default".to_string(),
        reading_expand_spoilers: false,
    })
}
