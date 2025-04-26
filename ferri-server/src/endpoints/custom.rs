use main::ap::http::HttpClient;
use rocket::{State, get, response::status};
use rocket_db_pools::Connection;

use uuid::Uuid;

use crate::{
    Db,
    types::{self, activity, content, webfinger},
};

#[get("/finger/<account>")]
pub async fn finger_account(mut db: Connection<Db>, account: &str) -> status::Accepted<String> {
    // user@host.com
    let (name, host) = account.split_once("@").unwrap();
    let user = resolve_user(name, host).await;

    // Make actor
    sqlx::query!(
        r#"
      INSERT INTO actor (id, inbox, outbox)
      VALUES (?1, ?2, ?3)
      ON CONFLICT(id) DO NOTHING
    "#,
        user.id,
        user.inbox,
        user.outbox
    )
    .execute(&mut **db)
    .await
    .unwrap();

    let uuid = Uuid::new_v4().to_string();
    let username = format!("{}@{}", user.name, host);
    sqlx::query!(
        r#"
      INSERT INTO user (id, username, actor_id, display_name)
      VALUES (?1, ?2, ?3, ?4)
      ON CONFLICT(actor_id) DO NOTHING
    "#,
        uuid,
        username,
        user.id,
        user.preferred_username
    )
    .execute(&mut **db)
    .await
    .unwrap();

    status::Accepted(format!("https://ferri.amy.mov/users/{}", uuid))
}

pub async fn resolve_user(acct: &str, host: &str) -> types::Person {
    let client = reqwest::Client::new();
    let url = format!(
        "https://{}/.well-known/webfinger?resource=acct:{}",
        host, acct
    );
    let wf = client
        .get(url)
        .send()
        .await
        .unwrap()
        .json::<webfinger::WebfingerResponse>()
        .await
        .unwrap();

    let actor_link = wf
        .links
        .iter()
        .find(|l| l.ty == Some("application/activity+json".to_string()))
        .unwrap();

    let href = actor_link.href.as_ref().unwrap();
    client
        .get(href)
        .header("Accept", "application/activity+json")
        .send()
        .await
        .unwrap()
        .json::<types::Person>()
        .await
        .unwrap()
}

#[get("/test")]
pub async fn test(http: &State<HttpClient>) -> &'static str {
    let user = resolve_user("amy@fedi.amy.mov", "fedi.amy.mov").await;

    let post = activity::CreateActivity {
        id: "https://ferri.amy.mov/activities/amy/20".to_string(),
        ty: "Create".to_string(),
        actor: "https://ferri.amy.mov/users/amy".to_string(),
        object: content::Post {
            context: "https://www.w3.org/ns/activitystreams".to_string(),
            id: "https://ferri.amy.mov/users/amy/posts/20".to_string(),
            ty: "Note".to_string(),
            content: "My first post".to_string(),
            ts: "2025-04-10T10:48:11Z".to_string(),
            to: vec!["https://ferri.amy.mov/users/amy/followers".to_string()],
            cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
            attributed_to: None
        },
        ts: "2025-04-10T10:48:11Z".to_string(),
        to: vec!["https://ferri.amy.mov/users/amy/followers".to_string()],
        cc: vec![],
    };

    let key_id = "https://ferri.amy.mov/users/amy#main-key";
    let follow = http
        .post(user.inbox)
        .json(&post)
        .sign(key_id)
        .activity()
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    "Hello, world!"
}
