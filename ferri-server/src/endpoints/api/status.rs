use crate::timeline::TimelineStatus;
use main::ap::{self, http::HttpClient};
use rocket::{
    FromForm, State,
    form::Form,
    get, post,
    serde::{Deserialize, Serialize, json::Json},
};
use rocket_db_pools::Connection;
use uuid::Uuid;

use crate::api::user::CredentialAcount;
use crate::{AuthenticatedUser, Db};

#[derive(Serialize, Deserialize, Debug, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct Status {
    status: String,
}

#[derive(Serialize, Deserialize, Debug, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct StatusContext {
    ancestors: Vec<Status>,
    descendants: Vec<Status>,
}

#[get("/statuses/<_status>/context")]
pub async fn status_context(
    _status: &str,
    _user: AuthenticatedUser,
    _db: Connection<Db>,
) -> Json<StatusContext> {
    Json(StatusContext {
        ancestors: vec![],
        descendants: vec![],
    })
}

async fn create_status(
    user: AuthenticatedUser,
    mut db: Connection<Db>,
    http: &HttpClient,
    status: &Status,
) -> TimelineStatus {
    let user = ap::User::from_id(&user.id, &mut **db).await.unwrap();
    let outbox = ap::Outbox::for_user(user.clone(), http);

    let post_id = ap::new_id();
    let now = ap::now();

    let post = ap::Post::from_parts(post_id, status.status.clone(), user.clone())
        .to(format!("{}/followers", user.uri()))
        .cc("https://www.w3.org/ns/activitystreams#Public".to_string());

    post.save(&mut **db).await;

    let actor = sqlx::query!(
        "SELECT * FROM actor WHERE id = ?1",
        "https://fedi.amy.mov/users/9zkygethkdw60001"
    )
    .fetch_one(&mut **db)
    .await
    .unwrap();

    let create_id = format!("https://ferri.amy.mov/activities/{}", Uuid::new_v4());

    let activity = ap::Activity {
        id: create_id,
        ty: ap::ActivityType::Create,
        object: post.clone().to_ap(),
        to: vec![format!("{}/followers", user.uri())],
        published: now,
        cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        ..Default::default()
    };

    let actor = ap::Actor::from_raw(actor.id.clone(), actor.inbox.clone(), actor.outbox.clone());

    let req = ap::OutgoingActivity {
        req: activity,
        signed_by: format!("{}#main-key", user.uri()),
        to: actor,
    };

    req.save(&mut **db).await;
    outbox.post(req).await;

    TimelineStatus {
        id: post.id().to_string(),
        created_at: post.created_at(),
        in_reply_to_id: None,
        in_reply_to_account_id: None,
        content: post.content().to_string(),
        visibility: "public".to_string(),
        spoiler_text: "".to_string(),
        sensitive: false,
        uri: post.uri(),
        url: post.uri(),
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
            id: user.id().to_string(),
            username: user.username().to_string(),
            acct: user.username().to_string(),
            display_name: user.display_name().to_string(),
            locked: false,
            bot: false,
            created_at: "2025-04-10T22:12:09Z".to_string(),
            attribution_domains: vec![],
            note: "".to_string(),
            url: user.uri(),
            avatar: "https://ferri.amy.mov/assets/pfp.png".to_string(),
            avatar_static: "https://ferri.amy.mov/assets/pfp.png".to_string(),
            header: "https://ferri.amy.mov/assets/pfp.png".to_string(),
            header_static: "https://ferri.amy.mov/assets/pfp.png".to_string(),
            followers_count: 1,
            following_count: 1,
            statuses_count: 1,
            last_status_at: "2025-04-10T22:14:34Z".to_string(),
        },
    }
}

#[post("/statuses", data = "<status>")]
pub async fn new_status(
    db: Connection<Db>,
    helpers: &State<crate::Helpers>,
    status: Form<Status>,
    user: AuthenticatedUser,
) -> Json<TimelineStatus> {
    Json(create_status(user, db, &helpers.http, &status).await)
}

#[post("/statuses", data = "<status>", rank = 2)]
pub async fn new_status_json(
    db: Connection<Db>,
    helpers: &State<crate::Helpers>,
    status: Json<Status>,
    user: AuthenticatedUser,
) -> Json<TimelineStatus> {
    Json(create_status(user, db, &helpers.http, &status).await)
}
