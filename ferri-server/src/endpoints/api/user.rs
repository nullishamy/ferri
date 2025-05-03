use main::ap;
use rocket::response::status::NotFound;
use rocket::{
    State, get, post,
    serde::{Deserialize, Serialize, json::Json},
};
use rocket_db_pools::Connection;
use uuid::Uuid;

use crate::timeline::{TimelineAccount, TimelineStatus};
use crate::{AuthenticatedUser, Db};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CredentialAcount {
    pub id: String,
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
    pub last_status_at: String,
}

#[get("/accounts/verify_credentials")]
pub async fn verify_credentials() -> Json<CredentialAcount> {
    Json(CredentialAcount {
        id: "9b9d497b-2731-435f-a929-e609ca69dac9".to_string(),
        username: "amy".to_string(),
        acct: "amy@ferri.amy.mov".to_string(),
        display_name: "amy".to_string(),
        locked: false,
        bot: false,
        created_at: "2025-04-10T22:12:09Z".to_string(),
        attribution_domains: vec![],
        note: "".to_string(),
        url: "https://ferri.amy.mov/@amy".to_string(),
        avatar: "https://ferri.amy.mov/assets/pfp.png".to_string(),
        avatar_static: "https://ferri.amy.mov/assets/pfp.png".to_string(),
        header: "https://ferri.amy.mov/assets/pfp.png".to_string(),
        header_static: "https://ferri.amy.mov/assets/pfp.png".to_string(),
        followers_count: 1,
        following_count: 1,
        statuses_count: 1,
        last_status_at: "2025-04-10T22:14:34Z".to_string(),
    })
}

#[post("/accounts/<uuid>/follow")]
pub async fn new_follow(
    mut db: Connection<Db>,
    helpers: &State<crate::Helpers>,
    uuid: &str,
    user: AuthenticatedUser,
) -> Result<(), NotFound<String>> {
    let http = &helpers.http;

    let follower = ap::User::from_actor_id(&user.actor_id.0, &mut **db).await;

    let followed = ap::User::from_id(uuid, &mut **db)
        .await
        .map_err(|e| NotFound(e.to_string()))?;

    let outbox = ap::Outbox::for_user(follower.clone(), http);

    let activity = ap::Activity {
        id: format!("https://ferri.amy.mov/activities/{}", Uuid::new_v4()),
        ty: ap::ActivityType::Follow,
        object: followed.actor_id().to_string(),
        ..Default::default()
    };

    let req = ap::OutgoingActivity {
        signed_by: format!("{}#main-key", follower.uri()),
        req: activity,
        to: followed.actor().clone(),
    };

    req.save(&mut **db).await;
    outbox.post(req).await;

    Ok(())
}

#[get("/accounts/<uuid>")]
pub async fn account(
    mut db: Connection<Db>,
    uuid: &str,
    _user: AuthenticatedUser,
) -> Result<Json<TimelineAccount>, NotFound<String>> {
    let user = ap::User::from_id(uuid, &mut **db)
        .await
        .map_err(|e| NotFound(e.to_string()))?;
    let user_uri = format!("https://ferri.amy.mov/users/{}", user.username());
    Ok(Json(CredentialAcount {
        id: user.id().to_string(),
        username: user.username().to_string(),
        acct: user.username().to_string(),
        display_name: user.display_name().to_string(),
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
    }))
}

#[get("/accounts/<uuid>/statuses?<_limit>")]
pub async fn statuses(
    mut db: Connection<Db>,
    uuid: &str,
    _limit: Option<i64>,
    _user: AuthenticatedUser,
) -> Result<Json<Vec<TimelineStatus>>, NotFound<String>> {
    let user = ap::User::from_id(uuid, &mut **db)
        .await
        .map_err(|e| NotFound(e.to_string()))?;

    let uid = user.id();
    let posts = sqlx::query!(
        r#"
            SELECT p.id as "post_id", u.id as "user_id", p.content, p.uri as "post_uri", u.username, u.display_name, u.actor_id, p.created_at 
            FROM post p
            INNER JOIN user u on p.user_id = u.id
            WHERE u.id = ?1
            ORDER BY p.created_at DESC
        "#, uid)
    .fetch_all(&mut **db)
    .await
    .unwrap();

    let mut out = Vec::<TimelineStatus>::new();
    for record in posts {
        let user_uri = format!("https://ferri.amy.mov/users/{}", record.username);
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
            reblog: None,
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

    Ok(Json(out))
}
