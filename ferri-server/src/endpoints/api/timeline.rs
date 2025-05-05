use crate::{AuthenticatedUser, Db, endpoints::api::user::CredentialAcount};
use main::types::{api, get, ObjectUuid};
use rocket::{
    get,
    serde::{Deserialize, Serialize, json::Json},
};
use rocket_db_pools::Connection;

pub type TimelineAccount = CredentialAcount;

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct TimelineStatusAttachment {
    id: ObjectUuid,
    #[serde(rename = "type")]
    ty: String,
    url: String,
    description: String
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct TimelineStatus {
    pub id: String,
    pub created_at: String,
    pub in_reply_to_id: Option<String>,
    pub in_reply_to_account_id: Option<String>,
    pub content: String,
    pub visibility: String,
    pub spoiler_text: String,
    pub sensitive: bool,
    pub uri: String,
    pub url: String,
    pub replies_count: i64,
    pub reblogs_count: i64,
    pub favourites_count: i64,
    pub favourited: bool,
    pub reblogged: bool,
    pub muted: bool,
    pub bookmarked: bool,
    pub reblog: Option<Box<TimelineStatus>>,
    pub media_attachments: Vec<TimelineStatusAttachment>,
    pub account: TimelineAccount,
}

#[get("/timelines/home")]
pub async fn home(
    mut db: Connection<Db>,
    user: AuthenticatedUser,
) -> Json<Vec<api::Status>> {
    let posts = get::home_timeline(user.actor_id, &mut **db)
        .await
        .unwrap();

    Json(posts.into_iter().map(Into::into).collect())
}
