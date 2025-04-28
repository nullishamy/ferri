use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::fmt::Debug;
use uuid::Uuid;

pub mod convert;
pub mod get;
pub mod make;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("an unknown error occured when creating: {0}")]
    CreationError(String),
    #[error("an unknown error occured when fetching: {0}")]
    FetchError(String)
}

pub const AS_CONTEXT_RAW: &'static str = "https://www.w3.org/ns/activitystreams";
pub fn as_context() -> ObjectContext {
    ObjectContext::Str(AS_CONTEXT_RAW.to_string())
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum ObjectContext {
    Str(String),
    Vec(Vec<serde_json::Value>),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct ObjectUri(pub String);

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct ObjectUuid(pub String);

impl ObjectUuid {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Object {
    #[serde(rename = "@context")]
    context: ObjectContext,
    id: ObjectUri,
}

pub mod db {
    use chrono::{DateTime, Utc};
    use super::*;

    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct Actor {
        pub id: ObjectUri,
        pub inbox: String,
        pub outbox: String,
    }

    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct UserPosts {
        // User may have no posts
        pub last_post_at: Option<DateTime<Utc>>
    }

    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct User {
        pub id: ObjectUuid,
        pub actor: Actor,
        pub username: String,
        pub display_name: String,
        pub acct: String,
        pub remote: bool,
        pub url: String,
        pub created_at: DateTime<Utc>,
        
        pub posts: UserPosts
    }
}

pub mod ap {
    use serde::{Serialize, Deserialize};
    use super::*;
    
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct Actor {
        #[serde(flatten)]
        pub obj: Object,
        
        pub inbox: String,
        pub outbox: String,
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct Person {
        #[serde(flatten)]
        pub obj: Object,

        pub following: String,
        pub followers: String,
        
        pub summary: String,
        pub inbox: String,
        pub outbox: String,
        
        pub preferred_username: String,
        pub name: String,
        
        pub public_key: Option<UserKey>,
    }
    
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct UserKey {
        pub id: String,
        pub owner: String,
        
        #[serde(rename = "publicKeyPem")]
        pub public_key: String,
    }
}

pub mod api {
    use serde::{Serialize, Deserialize};
    use super::*;
    
    // API will not really use actors so treat them as DB actors
    // until we require specificity
    pub type Actor = db::Actor;

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct Account {
        pub id: ObjectUuid,
        pub username: String,
        pub acct: String,
        pub display_name: String,
        
        pub locked: bool,
        pub bot: bool,
        
        pub created_at: String,
        pub attribution_domains: Vec<String>,
        
        pub note: String,
        pub url: String,
        
        pub avatar: String,
        pub avatar_static: String,
        pub header: String,
        pub header_static: String,
        
        pub followers_count: i64,
        pub following_count: i64,
        pub statuses_count: i64,
        pub last_status_at: Option<String>,
        
        pub emojis: Vec<Emoji>,
        pub fields: Vec<CustomField>,
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct Emoji {
        pub shortcode: String,
        pub url: String,
        pub static_url: String,
        pub visible_in_picker: bool,
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct CustomField {
        pub name: String,
        pub value: String,
        pub verified_at: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
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
    pub struct Configuration {
        pub urls: Urls,
        pub accounts: Accounts,
        pub statuses: Statuses,
        pub media_attachments: MediaAttachments,
        pub polls: Polls,
        pub translation: Translation,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Accounts {
        pub max_featured_tags: i64,
        pub max_pinned_statuses: i64,
    }

    #[derive(Debug, Serialize, Deserialize)]
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
    pub struct Polls {
        pub max_options: i64,
        pub max_characters_per_option: i64,
        pub min_expiration: i64,
        pub max_expiration: i64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Statuses {
        pub max_characters: i64,
        pub max_media_attachments: i64,
        pub characters_reserved_per_url: i64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Translation {
        pub enabled: bool,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Urls {
        pub streaming: String,
        pub about: String,
        pub privacy_policy: String,
        pub terms_of_service: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Contact {
        pub email: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Field {
        pub name: String,
        pub value: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Icon {
        pub src: String,
        pub size: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Registrations {
        pub enabled: bool,
        pub approval_required: bool,
        pub reason_required: bool,
        pub message: Option<String>,
        pub min_age: i64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Thumbnail {
        pub url: String,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn ap_actor_to_db() {
        let domain = "https://example.com";
        
        let ap = ap::Actor {
            obj: Object {
                context: as_context(),
                id: ObjectUri(format!("{}/users/sample", domain)),
            },
            inbox: format!("{}/users/sample/inbox", domain),
            outbox: format!("{}/users/sample/outbox", domain),
        };

        let db: db::Actor = ap.into();
        
        assert_eq!(db, db::Actor {
            id: ObjectUri("https://example.com/users/sample".to_string()),
            inbox: "https://example.com/users/sample/inbox".to_string(),
            outbox: "https://example.com/users/sample/outbox".to_string(),
        });
    }
}
