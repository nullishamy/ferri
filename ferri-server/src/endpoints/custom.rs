use crate::{Db, OutboundQueue};
use main::types::{ap, api};
use rocket::{State, get, response::status};
use rocket_db_pools::Connection;
use uuid::Uuid;

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
        user.obj.id.0,
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
        user.obj.id.0,
        user.preferred_username
    )
    .execute(&mut **db)
    .await
    .unwrap();

    status::Accepted(format!("https://ferri.amy.mov/users/{}", uuid))
}

pub async fn resolve_user(acct: &str, host: &str) -> ap::Person {
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
        .json::<api::WebfingerHit>()
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
        .json::<ap::Person>()
        .await
        .unwrap()
}

#[get("/test")]
pub async fn test(outbound: &State<OutboundQueue>, mut db: Connection<Db>) -> &'static str {
    use main::types::{ObjectUuid, api, get};
    outbound.0.send(main::federation::QueueMessage::Heartbeat).await;

    let id = ObjectUuid("9b9d497b-2731-435f-a929-e609ca69dac9".to_string());
    let user = dbg!(get::user_by_id(id, &mut db).await.unwrap());
    let apu: api::Account = user.into();
    dbg!(apu);

    "Hello, world!"
}
