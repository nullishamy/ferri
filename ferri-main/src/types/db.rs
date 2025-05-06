use super::*;
use chrono::{DateTime, Utc};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Follow {
    pub id: ObjectUri,
    pub follower: ObjectUri,
    pub followed: ObjectUri,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Actor {
    pub id: ObjectUri,
    pub inbox: String,
    pub outbox: String,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct UserPosts {
    // User may have no posts
    pub last_post_at: Option<DateTime<Utc>>,
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
    pub icon_url: String,

    pub posts: UserPosts,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Attachment {
    pub id: ObjectUuid,
    pub post_id: ObjectUuid,
    pub url: String,
    pub media_type: Option<String>,
    pub sensitive: bool,
    pub alt: Option<String>
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Post {
    pub id: ObjectUuid,
    pub uri: ObjectUri,
    pub user: User,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub boosted_post: Option<Box<Post>>,
    pub attachments: Vec<Attachment>
}

