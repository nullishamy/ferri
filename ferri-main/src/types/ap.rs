use super::*;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum ActivityType {
    Reject,
    Create,
    Note,
    Delete,
    Undo,
    Accept,
    Announce,
    Person,
    Like,
    Follow,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MinimalActivity {
    #[serde(flatten)]
    pub obj: Object,

    #[serde(rename = "type")]
    pub ty: ActivityType,
}

pub type DeleteActivity = BasicActivity;
pub type LikeActivity = BasicActivity;

#[derive(Serialize, Deserialize, Debug)]
pub struct BasicActivity {
    #[serde(flatten)]
    pub obj: Object,

    pub object: String,
    pub actor: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateActivity {
    #[serde(flatten)]
    pub obj: Object,

    #[serde(rename = "type")]
    pub ty: ActivityType,

    pub object: Post,
    pub actor: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,

    #[serde(rename = "published")]
    pub ts: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FollowActivity {
    #[serde(flatten)]
    pub obj: Object,

    #[serde(rename = "type")]
    pub ty: ActivityType,

    pub object: String,
    pub actor: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AcceptActivity {
    #[serde(flatten)]
    pub obj: Object,
    
    #[serde(rename = "type")]
    pub ty: ActivityType,

    pub object: String,
    pub actor: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoostActivity {
    #[serde(flatten)]
    pub obj: Object,

    #[serde(rename = "type")]
    pub ty: ActivityType,

    pub actor: String,
    pub published: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub object: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PostAttachmentType {
    Document
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PostAttachment {
    #[serde(rename = "type")]
    pub ty: PostAttachmentType,
    
    pub media_type: String,
    pub url: String,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub name: String,
    
    pub summary: Option<String>,
    #[serde(default)]
    pub sensitive: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    #[serde(flatten)]
    pub obj: Object,

    #[serde(rename = "type")]
    pub ty: ActivityType,

    #[serde(rename = "published")]
    pub ts: String,
    pub content: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,

    pub attachment: Vec<PostAttachment>,

    #[serde(rename = "attributedTo")]
    pub attributed_to: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Actor {
    #[serde(flatten)]
    pub obj: Object,

    pub inbox: String,
    pub outbox: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum IconType {
    Image
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct PersonIcon {
    #[serde(rename = "type")]
    pub ty: IconType,
    pub url: String,

    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(flatten)]
    pub obj: Object,

    #[serde(rename = "type")]
    pub ty: ActivityType,

    pub following: String,
    pub followers: String,

    pub summary: String,
    pub inbox: String,
    pub outbox: String,

    pub preferred_username: String,
    pub name: String,

    pub public_key: Option<UserKey>,

    pub icon: Option<PersonIcon>
}

pub struct RemoteInfo {
    pub is_remote: bool,
    pub web_url: String,
    pub acct: String
}

impl Person {
    pub fn remote_info(&self) -> RemoteInfo {
        let url = Url::parse(&self.obj.id.0).unwrap();
        let host = url.host_str().unwrap();

        let (acct, remote) = if host != "ferri.amy.mov" {
            (format!("{}@{}", self.preferred_username, host), true)
        } else {
            (self.preferred_username.clone(), false)
        };

        let url = format!("https://ferri.amy.mov/{}", acct);

        RemoteInfo {
            acct: acct.to_string(),
            web_url: url,
            is_remote: remote,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct UserKey {
    pub id: String,
    pub owner: String,

    #[serde(rename = "publicKeyPem")]
    pub public_key: String,
}
