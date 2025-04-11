use crate::{Db, endpoints::api::user::CredentialAcount};
use rocket::{
    get,
    serde::{Deserialize, Serialize, json::Json},
};
use rocket_db_pools::Connection;

pub type TimelineAccount = CredentialAcount;

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct TimelineStatus {
    id: String,
    created_at: String,
    in_reply_to_id: Option<String>,
    in_reply_to_account_id: Option<String>,
    content: String,
    visibility: String,
    spoiler_text: String,
    sensitive: bool,
    uri: String,
    url: String,
    replies_count: i64,
    reblogs_count: i64,
    favourites_count: i64,
    favourited: bool,
    reblogged: bool,
    muted: bool,
    bookmarked: bool,
    media_attachments: Vec<()>,
    account: TimelineAccount,
}

#[get("/timelines/home?<limit>")]
pub async fn home(mut db: Connection<Db>, limit: i64) -> Json<Vec<TimelineStatus>> {
    let posts = sqlx::query!(
        r#"
        SELECT p.id as "post_id", u.id as "user_id", p.content, u.username, u.display_name, u.actor_id FROM post p
        INNER JOIN user u on p.user_id = u.id
    "#
    )
    .fetch_all(&mut **db)
    .await
    .unwrap();

    let mut out = Vec::<TimelineStatus>::new();
    for record in posts {
        out.push(TimelineStatus {
            id: record.post_id.clone(),
            created_at: "2025-04-10T22:12:09Z".to_string(),
            in_reply_to_id: None,
            in_reply_to_account_id: None,
            content: record.content.clone(),
            visibility: "public".to_string(),
            spoiler_text: "".to_string(),
            sensitive: false,
            uri: record.post_id.clone(),
            url: record.post_id.clone(),
            replies_count: 0,
            reblogs_count: 0,
            favourites_count: 0,
            favourited: false,
            reblogged: false,
            muted: false,
            bookmarked: false,
            media_attachments: vec![],
            account: CredentialAcount {
                id: record.actor_id.clone(),
                username: record.username.clone(),
                acct: record.username.clone(),
                display_name: record.display_name.clone(),
                locked: false,
                bot: false,
                created_at: "2025-04-10T22:12:09Z".to_string(),
                attribution_domains: vec![],
                note: "".to_string(),
                url: record.actor_id.clone(),
                avatar: "https://ferri.amy.mov/assets/pfp.png".to_string(),
                avatar_static: "https://ferri.amy.mov/assets/pfp.png".to_string(),
                header: "https://ferri.amy.mov/assets/pfp.png".to_string(),
                header_static: "https://ferri.amy.mov/assets/pfp.png".to_string(),
                followers_count: 1,
                following_count: 1,
                statuses_count: 1,
                last_status_at: "2025-04-10T22:14:34Z".to_string(),
            },
        });
    }
    
    Json(out)
}
