use main::ap;
use rocket::{
    State, get, post,
    serde::{Deserialize, Serialize, json::Json},
};
use rocket_db_pools::Connection;
use uuid::Uuid;

use crate::{AuthenticatedUser, Db, http::HttpClient};

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
        id: "https://ferri.amy.mov/users/amy".to_string(),
        username: "amy".to_string(),
        acct: "amy@ferri.amy.mov".to_string(),
        display_name: "amy".to_string(),
        locked: false,
        bot: false,
        created_at: "2025-04-10T22:12:09Z".to_string(),
        attribution_domains: vec![],
        note: "".to_string(),
        url: "https://ferri.amy.mov/@amy".to_string(),
        avatar: "https://i.sstatic.net/l60Hf.png".to_string(),
        avatar_static: "https://i.sstatic.net/l60Hf.png".to_string(),
        header: "https://i.sstatic.net/l60Hf.png".to_string(),
        header_static: "https://i.sstatic.net/l60Hf.png".to_string(),
        followers_count: 1,
        following_count: 1,
        statuses_count: 1,
        last_status_at: "2025-04-10T22:14:34Z".to_string(),
    })
}

#[post("/accounts/<account>/follow")]
pub async fn new_follow(
    mut db: Connection<Db>,
    http: &State<HttpClient>,
    account: &str,
    user: AuthenticatedUser,
) {
    let follower = ap::User::from_actor_id(&user.actor_id, &mut **db).await;
    let followed = ap::User::from_username(account, &mut **db).await;

    let outbox = ap::Outbox::for_user(follower.clone(), http.inner());

    let activity = ap::Activity {
        id: format!("https://ferri.amy.mov/activities/{}", Uuid::new_v4()),
        ty: ap::ActivityType::Follow,
        object: followed.actor_id().to_string(),
        ..Default::default()
    };

    let req = ap::OutgoingActivity {
        signed_by: format!(
            "https://ferri.amy.mov/users/{}#main-key",
            follower.username()
        ),
        req: activity,
        to: followed.actor().clone(),
    };

    req.save(&mut **db).await;
    outbox.post(req).await;
}
