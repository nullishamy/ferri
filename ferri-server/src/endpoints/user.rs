use rocket::{
    Responder, State, get,
    http::ContentType,
    response::{Redirect, status::NotFound},
    serde::json::Json,
};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};

use main::types::{ap, as_context, get, Object, ObjectContext, ObjectUri, ObjectUuid};

use super::activity_type;
use crate::Db;

#[derive(Serialize, Deserialize)]
pub struct OrderedCollection {
    #[serde(rename = "@context")]
    context: ObjectContext,
    #[serde(rename = "type")]
    ty: String,
    id: String,
    total_items: i64,
    ordered_items: Vec<String>,
}

#[get("/users/<user>/inbox")]
pub async fn inbox(user: String) -> Json<OrderedCollection> {
    Json(OrderedCollection {
        context: as_context(),
        ty: "OrderedCollection".to_string(),
        id: format!("https://ferri.amy.mov/users/{}/inbox", user),
        total_items: 0,
        ordered_items: vec![],
    })
}

#[get("/users/<user>/outbox")]
pub async fn outbox(user: String) -> Json<OrderedCollection> {
    Json(OrderedCollection {
        context: as_context(),
        ty: "OrderedCollection".to_string(),
        id: format!("https://ferri.amy.mov/users/{}/outbox", user),
        total_items: 0,
        ordered_items: vec![],
    })
}

#[get("/users/<uuid>/followers")]
pub async fn followers(
    mut db: Connection<Db>,
    uuid: &str,
) -> Result<ActivityResponse<Json<OrderedCollection>>, NotFound<String>> {
    let user = get::user_by_id(ObjectUuid(uuid.to_string()), &mut **db)
        .await
        .unwrap();
    
    let followers = get::followers_for_user(user.id.clone(), &mut **db)
        .await
        .unwrap();
    
    ap_ok(Json(OrderedCollection {
        context: as_context(),
        ty: "OrderedCollection".to_string(),
        total_items: 1,
        id: format!("https://ferri.amy.mov/users/{}/followers", uuid),
        ordered_items: followers
            .into_iter()
            .map(|f| f.follower.0)
            .collect(),
    }))
}

#[get("/users/<uuid>/following")]
pub async fn following(
    mut db: Connection<Db>,
    uuid: &str,
) -> Result<ActivityResponse<Json<OrderedCollection>>, NotFound<String>> {
    let user = get::user_by_id(ObjectUuid(uuid.to_string()), &mut **db)
        .await
        .unwrap();
    
    let followers = get::following_for_user(user.id.clone(), &mut **db)
        .await
        .unwrap();
    
    ap_ok(Json(OrderedCollection {
        context: as_context(),
        ty: "OrderedCollection".to_string(),
        total_items: 1,
        id: format!("https://ferri.amy.mov/users/{}/following", uuid),
        ordered_items: followers
            .into_iter()
            .map(|f| f.followed.0)
            .collect(),
    }))
}

#[get("/users/<uuid>/posts/<post>")]
pub async fn post(
    mut db: Connection<Db>,
    helpers: &State<crate::Helpers>,
    uuid: &str,
    post: String,
) -> (ContentType, Json<ap::Post>) {
    let config = &helpers.config;
    let post = sqlx::query!(
        r#"
        SELECT * FROM post WHERE id = ?1
    "#,
        post
    )
    .fetch_one(&mut **db)
    .await
    .unwrap();

    (
        activity_type(),
        Json(ap::Post {
            obj: Object {
                context: as_context(),
                id: ObjectUri(config.post_url(uuid, &post.id)),
            },
            attachment: vec![],
            attributed_to: Some(config.user_url(uuid)),
            ty: ap::ActivityType::Note,
            content: post.content,
            ts: post.created_at,
            to: vec![config.followers_url(uuid)],
            cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        }),
    )
}

#[derive(Debug, Responder)]
pub enum UserFetchError {
    NotFound(NotFound<String>),
    Moved(Redirect),
}

type ActivityResponse<T> = (ContentType, T);
fn ap_response<T>(t: T) -> ActivityResponse<T> {
    (activity_type(), t)
}

fn ap_ok<T, E>(t: T) -> Result<ActivityResponse<T>, E> {
    Ok(ap_response(t))
}

#[get("/users/<uuid>")]
pub async fn user(
    mut db: Connection<Db>,
    uuid: &str,
) -> Result<ActivityResponse<Json<ap::Person>>, UserFetchError> {
    if uuid == "amy" {
        return Err(UserFetchError::Moved(Redirect::permanent(
            "https://ferri.amy.mov/users/9b9d497b-2731-435f-a929-e609ca69dac9",
        )));
    }

    let user = get::user_by_id(ObjectUuid(uuid.to_string()), &mut db)
        .await
        .map_err(|e| UserFetchError::NotFound(NotFound(e.to_string())))?;

    ap_ok(Json(user.into()))
}
