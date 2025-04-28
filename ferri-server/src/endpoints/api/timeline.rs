use crate::{AuthenticatedUser, Db, endpoints::api::user::CredentialAcount, Config};
use rocket::{
    State,
    get,
    serde::{Deserialize, Serialize, json::Json},
};
use rocket_db_pools::Connection;

pub type TimelineAccount = CredentialAcount;

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
    pub media_attachments: Vec<()>,
    pub account: TimelineAccount,
}

#[get("/timelines/home")]
pub async fn home(
    mut db: Connection<Db>,
    config: &State<Config>,
    _user: AuthenticatedUser,
) -> Json<Vec<TimelineStatus>> {
    let posts = sqlx::query!(
        r#"
            SELECT p.id as "post_id", u.id as "user_id", p.content, p.uri as "post_uri", 
                u.username, u.display_name, u.actor_id, p.created_at, p.boosted_post_id
            FROM post p
            INNER JOIN user u on p.user_id = u.id
            ORDER BY datetime(p.created_at) DESC
        "#
    )
    .fetch_all(&mut **db)
    .await
    .unwrap();

    let mut out = Vec::<TimelineStatus>::new();
    for record in posts {
        let mut boost: Option<Box<TimelineStatus>> = None;
        if let Some(boosted_id) = record.boosted_post_id {
            let record = sqlx::query!(
               r#"
                 SELECT p.id as "post_id", u.id as "user_id", p.content, p.uri as "post_uri", 
                   u.username, u.display_name, u.actor_id, p.created_at, p.boosted_post_id
                 FROM post p
                 INNER JOIN user u on p.user_id = u.id
                 WHERE p.id = ?1
               "#, boosted_id)
                .fetch_one(&mut **db)
                .await
                .unwrap();
            
            let user_uri = config.user_url(&record.user_id);
            boost = Some(Box::new(TimelineStatus {
                id: record.post_id.clone(),
                created_at: record.created_at.clone(),
                in_reply_to_id: None,
                in_reply_to_account_id: None,
                content: record.content.clone(),
                visibility: "public".to_string(),
                spoiler_text: "".to_string(),
                sensitive: false,
                uri: record.post_uri.clone(),
                url: record.post_uri.clone(),
                replies_count: 0,
                reblogs_count: 0,
                favourites_count: 0,
                favourited: false,
                reblogged: false,
                reblog: boost,
                muted: false,
                bookmarked: false,
                media_attachments: vec![],
                account: CredentialAcount {
                    id: record.user_id.clone(),
                    username: record.username.clone(),
                    acct: record.username.clone(),
                    display_name: record.display_name.clone(),
                    locked: false,
                    bot: false,
                    created_at: "2025-04-10T22:12:09Z".to_string(),
                    attribution_domains: vec![],
                    note: "".to_string(),
                    url: user_uri,
                    avatar: "https://ferri.amy.mov/assets/pfp.png".to_string(),
                    avatar_static: "https://ferri.amy.mov/assets/pfp.png".to_string(),
                    header: "https://ferri.amy.mov/assets/pfp.png".to_string(),
                    header_static: "https://ferri.amy.mov/assets/pfp.png".to_string(),
                    followers_count: 1,
                    following_count: 1,
                    statuses_count: 1,
                    last_status_at: "2025-04-10T22:14:34Z".to_string(),
                },                
            }))
        }
        
        let user_uri = config.user_web_url(&record.username);
        out.push(TimelineStatus {
            id: record.post_id.clone(),
            created_at: record.created_at.clone(),
            in_reply_to_id: None,
            in_reply_to_account_id: None,
            content: record.content.clone(),
            visibility: "public".to_string(),
            spoiler_text: "".to_string(),
            sensitive: false,
            uri: record.post_uri.clone(),
            url: record.post_uri.clone(),
            replies_count: 0,
            reblogs_count: 0,
            favourites_count: 0,
            favourited: false,
            reblogged: false,
            reblog: boost,
            muted: false,
            bookmarked: false,
            media_attachments: vec![],
            account: CredentialAcount {
                id: record.user_id.clone(),
                username: record.username.clone(),
                acct: record.username.clone(),
                display_name: record.display_name.clone(),
                locked: false,
                bot: false,
                created_at: "2025-04-10T22:12:09Z".to_string(),
                attribution_domains: vec![],
                note: "".to_string(),
                url: user_uri,
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
