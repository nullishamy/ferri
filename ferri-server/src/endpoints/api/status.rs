use main::{config::Config, federation::{outbox::OutboxRequest, QueueMessage}, types::{api, make, ObjectUri, ObjectUuid}};
use rocket::{
    FromForm, State,
    form::Form,
    get, post,
    serde::{Deserialize, Serialize, json::Json},
};
use rocket_db_pools::Connection;
use main::types::db;

use crate::{AuthenticatedUser, Db, OutboundQueue};

#[derive(Serialize, Deserialize, Debug, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct StatusContext {
    ancestors: Vec<CreateStatus>,
    descendants: Vec<CreateStatus>,
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

#[derive(Serialize, Deserialize, Debug, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct CreateStatus {
    status: String,
}

fn to_db_post(req: &CreateStatus, user: &AuthenticatedUser, config: &Config) -> db::Post {
    let post_id = main::new_id();
    
    db::Post {
        id: ObjectUuid(post_id.clone()),
        uri: ObjectUri(config.post_url(&user.id.0, &post_id)),
        user: user.user.clone(),
        content: req.status.clone(),
        created_at: main::ap::now(),
        boosted_post: None,
        attachments: vec![]
    }
}

#[post("/statuses", data = "<status>")]
pub async fn new_status(
    mut db: Connection<Db>,
    helpers: &State<crate::Helpers>,
    status: Form<CreateStatus>,
    user: AuthenticatedUser,
) -> Json<api::Status> {
    let post = make::new_post(
        to_db_post(&status, &user, &helpers.config),
        &mut **db
    )
        .await
        .unwrap();

    Json(post.into())
} 

#[post("/statuses", data = "<status>", rank = 2)]
pub async fn new_status_json(
    mut db: Connection<Db>,
    helpers: &State<crate::Helpers>,
    outbound: &State<OutboundQueue>,
    status: Json<CreateStatus>,
    user: AuthenticatedUser,
) -> Json<api::Status> {
    let post = make::new_post(
        to_db_post(&status, &user, &helpers.config),
        &mut **db
    )
        .await
        .unwrap();


    let key_id = "https://ferri.amy.mov/users/9b9d497b-2731-435f-a929-e609ca69dac9#main-key";
    outbound.0.send(QueueMessage::Outbound(
        OutboxRequest::Status(post.clone(), key_id.to_string()))
    )
    .await;
    
    Json(post.into())
}
