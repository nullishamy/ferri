use main::federation::outbox::OutboxRequest;
use main::federation::QueueMessage;
use main::types::{api, get, ObjectUuid};
use rocket::response::status::NotFound;
use rocket::{
    State, get, post,
    serde::{Deserialize, Serialize, json::Json},
};
use rocket_db_pools::Connection;
use tracing::info;

use crate::{AuthenticatedUser, Db, OutboundQueue};

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
pub async fn verify_credentials(user: AuthenticatedUser) -> Json<CredentialAcount> {
    info!("verifying creds for {:#?}", user);
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
    outbound: &State<OutboundQueue>,
    uuid: &str,
    user: AuthenticatedUser,
) -> Result<(), NotFound<String>> {
    let follower = user.user;
    let followed = get::user_by_id(ObjectUuid(uuid.to_string()), &mut **db)
        .await
        .unwrap();

    let conn = db.into_inner();
    let conn = conn.detach();

    let msg = QueueMessage::Outbound(OutboxRequest::Follow {
        follower,
        followed,
        conn
    });
    
    outbound.0.send(msg).await;
    
    Ok(())
}

#[get("/accounts/relationships?<id>")]
pub async fn relationships(
    id: Vec<String>,
    user: AuthenticatedUser
) -> Result<Json<Vec<api::Relationship>>, ()> {
    info!("{} looking up relationships for {:#?}", user.username, id);
    Ok(Json(vec![]))
}

#[get("/accounts/<uuid>")]
pub async fn account(
    mut db: Connection<Db>,
    uuid: &str,
    _user: AuthenticatedUser,
) -> Result<Json<api::Account>, NotFound<String>> {
    let user = get::user_by_id(ObjectUuid(uuid.to_string()), &mut **db)
        .await
        .map_err(|e| NotFound(e.to_string()))?;

    Ok(Json(user.into()))
}

#[get("/accounts/<uuid>/statuses?<_limit>")]
pub async fn statuses(
    mut db: Connection<Db>,
    uuid: &str,
    _limit: Option<i64>,
    _user: AuthenticatedUser,
) -> Result<Json<Vec<api::Status>>, NotFound<String>> {
    let user = get::user_by_id(ObjectUuid(uuid.to_string()), &mut **db)
        .await
        .map_err(|e| NotFound(e.to_string()))?;

    let posts = get::posts_for_user_id(user.id, &mut **db)
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.into())
        .collect();
    
    Ok(Json(posts))
}
