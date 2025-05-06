use super::*;
use serde::{Deserialize, Serialize};

// API will not really use actors so treat them as DB actors
// until we require specificity
pub type Actor = db::Actor;

#[derive(Serialize, Deserialize, Debug)]
pub struct CredentialApplication {
    pub name: String,
    pub scopes: String,
    pub redirect_uris: Vec<String>,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WebfingerLink {
    pub rel: String,
    #[serde(rename = "type")]
    pub ty: Option<String>,
    pub href: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WebfingerHit {
    pub subject: String,
    pub aliases: Vec<String>,
    pub links: Vec<WebfingerLink>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct StatusAttachment {
    pub id: ObjectUuid,
    #[serde(rename = "type")]
    pub ty: String,
    
    pub url: String,
    pub description: String
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Status {
    pub id: ObjectUuid,
    pub created_at: String,
    pub in_reply_to_id: Option<ObjectUri>,
    pub in_reply_to_account_id: Option<ObjectUri>,
    pub sensitive: bool,
    pub spoiler_text: String,
    pub visibility: String,
    pub language: String,
    pub uri: ObjectUri,
    pub url: String,
    pub replies_count: i64,
    pub reblogs_count: i64,
    pub favourites_count: i64,
    pub favourited: bool,
    pub reblogged: bool,
    pub muted: bool,
    pub bookmarked: bool,
    pub content: String,
    pub reblog: Option<Box<Status>>,
    pub application: Option<()>,
    pub account: Account,
    pub media_attachments: Vec<StatusAttachment>,
    pub mentions: Vec<Option<()>>,
    pub tags: Vec<Option<()>>,
    pub emojis: Vec<Option<()>>,
    pub card: Option<()>,
    pub poll: Option<()>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Relationship {
    id: ObjectUuid,
    following: bool,
    showing_reblogs: bool,
    notifying: bool,
    followed_by: bool,
    blocking: bool,
    blocked_by: bool,
    muting: bool,
    muting_notifications: bool,
    requested: bool,
    requested_by: bool,
    domain_blocking: bool,
    endorsed: bool,
    note: String
}

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

#[derive(Serialize, Deserialize)]
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
