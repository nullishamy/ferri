use chrono::Local;
use main::ap::{self, http::HttpClient};
use rocket::{
    FromForm, State,
    form::Form,
    post,
    serde::{Deserialize, Serialize, json::Json},
};
use rocket_db_pools::Connection;
use uuid::Uuid;
use crate::timeline::TimelineStatus;

use crate::{AuthenticatedUser, Db, types::content};
use crate::api::user::CredentialAcount;

#[derive(Serialize, Deserialize, Debug, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct Status {
    status: String,
}

#[post("/statuses", data = "<status>")]
pub async fn new_status(
    mut db: Connection<Db>,
    http: &State<HttpClient>,
    status: Form<Status>,
    user: AuthenticatedUser,
) {
    let user = ap::User::from_actor_id(&user.actor_id, &mut **db).await;
    let outbox = ap::Outbox::for_user(user.clone(), http);

    let post_id = Uuid::new_v4().to_string();

    let uri = format!(
        "https://ferri.amy.mov/users/{}/posts/{}",
        user.username(),
        post_id
    );
    let id = user.id();
    let now = Local::now().to_rfc3339();

    let post = sqlx::query!(
        r#"
            INSERT INTO post (id, uri, user_id, content, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            RETURNING *
        "#,
        post_id,
        uri,
        id,
        status.status,
        now
    )
    .fetch_one(&mut **db)
    .await
    .unwrap();

    let actors = sqlx::query!("SELECT * FROM actor")
        .fetch_all(&mut **db)
        .await
        .unwrap();

    for record in actors {
        // Don't send to ourselves
        if &record.id == user.actor_id() {
            continue
        }

        let create_id = format!("https://ferri.amy.mov/activities/{}", Uuid::new_v4());

        let activity = ap::Activity {
            id: create_id,
            ty: ap::ActivityType::Create,
            object: content::Post {
                context: "https://www.w3.org/ns/activitystreams".to_string(),
                id: uri.clone(),
                content: status.status.clone(),
                ty: "Note".to_string(),
                ts: Local::now().to_rfc3339(),
                to: vec![format!(
                    "https://ferri.amy.mov/users/{}/followers",
                    user.username()
                )],
                cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
            },
            to: vec![format!(
                "https://ferri.amy.mov/users/{}/followers",
                user.username()
            )],
            cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
            ..Default::default()
        };

        let actor = ap::Actor::from_raw(
            record.id.clone(),
            record.inbox.clone(),
            record.outbox.clone(),
        );
        let req = ap::OutgoingActivity {
            req: activity,
            signed_by: format!("https://ferri.amy.mov/users/{}#main-key", user.username()),
            to: actor,
        };

        req.save(&mut **db).await;
        outbox.post(req).await;
    }
}

#[post("/statuses", data = "<status>", rank = 2)]
pub async fn new_status_json(
    mut db: Connection<Db>,
    http: &State<HttpClient>,
    status: Json<Status>,
    user: AuthenticatedUser,
) -> Json<TimelineStatus> {
    dbg!(&user);
    let user = ap::User::from_id(&user.username, &mut **db).await;
    let outbox = ap::Outbox::for_user(user.clone(), http);

    let post_id = Uuid::new_v4().to_string();

    let uri = format!(
        "https://ferri.amy.mov/users/{}/posts/{}",
        user.id(),
        post_id
    );
    let id = user.id();
    let now = Local::now().to_rfc3339();

    let post = sqlx::query!(
        r#"
            INSERT INTO post (id, uri, user_id, content, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            RETURNING *
        "#,
        post_id,
        uri,
        id,
        status.status,
        now
    )
    .fetch_one(&mut **db)
    .await
    .unwrap();

    let actors = sqlx::query!("SELECT * FROM actor")
        .fetch_all(&mut **db)
        .await
        .unwrap();

    for record in actors {
        // Don't send to ourselves
        if &record.id == user.actor_id() {
            continue
        }

        let create_id = format!("https://ferri.amy.mov/activities/{}", Uuid::new_v4());

        let activity = ap::Activity {
            id: create_id,
            ty: ap::ActivityType::Create,
            object: content::Post {
                context: "https://www.w3.org/ns/activitystreams".to_string(),
                id: uri.clone(),
                content: status.status.clone(),
                ty: "Note".to_string(),
                ts: Local::now().to_rfc3339(),
                to: vec![format!(
                    "https://ferri.amy.mov/users/{}/followers",
                    user.username()
                )],
                cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
            },
            to: vec![format!(
                "https://ferri.amy.mov/users/{}/followers",
                user.username()
            )],
            cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
            ..Default::default()
        };

        let actor = ap::Actor::from_raw(
            record.id.clone(),
            record.inbox.clone(),
            record.outbox.clone(),
        );
        let req = ap::OutgoingActivity {
            req: activity,
            signed_by: format!("https://ferri.amy.mov/users/{}#main-key", user.username()),
            to: actor,
        };

        req.save(&mut **db).await;
        outbox.post(req).await;
    }

    let user_uri = format!(
        "https://ferri.amy.mov/users/{}",
        user.id(),
    );
    Json(TimelineStatus {
        id: post.id.clone(),
        created_at: post.created_at.clone(),
        in_reply_to_id: None,
        in_reply_to_account_id: None,
        content: post.content.clone(),
        visibility: "public".to_string(),
        spoiler_text: "".to_string(),
        sensitive: false,
        uri: post.uri.clone(),
        url: post.uri.clone(),
        replies_count: 0,
        reblogs_count: 0,
        favourites_count: 0,
        favourited: false,
        reblogged: false,
        muted: false,
        bookmarked: false,
        media_attachments: vec![],
        account: CredentialAcount {
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
        }
    })
}
