use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Instance {
    pub domain: String,
    pub title: String,
    pub version: String,
    pub source_url: String,
    pub description: String,
    pub thumbnail: Thumbnail,
    pub icon: Vec<Icon>,
    pub languages: Vec<String>,
    pub configuration: Configuration,
    pub registrations: Registrations,
    pub contact: Contact,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Configuration {
    pub urls: Urls,
    pub accounts: Accounts,
    pub statuses: Statuses,
    pub media_attachments: MediaAttachments,
    pub polls: Polls,
    pub translation: Translation,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Accounts {
    pub max_featured_tags: i64,
    pub max_pinned_statuses: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MediaAttachments {
    pub supported_mime_types: Vec<String>,
    pub description_limit: i64,
    pub image_size_limit: i64,
    pub image_matrix_limit: i64,
    pub video_size_limit: i64,
    pub video_frame_rate_limit: i64,
    pub video_matrix_limit: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Polls {
    pub max_options: i64,
    pub max_characters_per_option: i64,
    pub min_expiration: i64,
    pub max_expiration: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Statuses {
    pub max_characters: i64,
    pub max_media_attachments: i64,
    pub characters_reserved_per_url: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Translation {
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Urls {
    pub streaming: String,
    pub about: String,
    pub privacy_policy: String,
    pub terms_of_service: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Contact {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Field {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Icon {
    pub src: String,
    pub size: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Registrations {
    pub enabled: bool,
    pub approval_required: bool,
    pub reason_required: bool,
    pub message: Option<String>,
    pub min_age: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Thumbnail {
    pub url: String,
}
