use rocket::{
    Responder, State, get,
    http::ContentType,
    response::{Redirect, status::NotFound},
    serde::json::Json,
};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};

use main::types::{Object, ObjectUri, ObjectUuid, ap, as_context, get};

use super::activity_type;
use crate::Db;

#[derive(Serialize, Deserialize)]
pub struct OrderedCollection {
    ty: String,
    total_items: i64,
    ordered_items: Vec<String>,
}

#[get("/users/<_user>/inbox")]
pub async fn inbox(_user: String) -> Json<OrderedCollection> {
    Json(OrderedCollection {
        ty: "OrderedCollection".to_string(),
        total_items: 0,
        ordered_items: vec![],
    })
}

#[get("/users/<_user>/outbox")]
pub async fn outbox(_user: String) -> Json<OrderedCollection> {
    Json(OrderedCollection {
        ty: "OrderedCollection".to_string(),
        total_items: 0,
        ordered_items: vec![],
    })
}

#[get("/users/<uuid>/followers")]
pub async fn followers(
    mut db: Connection<Db>,
    uuid: &str,
) -> Result<Json<OrderedCollection>, NotFound<String>> {
    let target = main::ap::User::from_id(uuid, &mut **db)
        .await
        .map_err(|e| NotFound(e.to_string()))?;

    let actor_id = target.actor_id();

    let followers = sqlx::query!(
        r#"
            SELECT follower_id FROM follow
            WHERE followed_id = ?
        "#,
        actor_id
    )
    .fetch_all(&mut **db)
    .await
    .unwrap();

    Ok(Json(OrderedCollection {
        ty: "OrderedCollection".to_string(),
        total_items: 1,
        ordered_items: followers
            .into_iter()
            .map(|f| f.follower_id)
            .collect::<Vec<_>>(),
    }))
}

#[get("/users/<uuid>/following")]
pub async fn following(
    mut db: Connection<Db>,
    uuid: &str,
) -> Result<Json<OrderedCollection>, NotFound<String>> {
    let target = main::ap::User::from_id(uuid, &mut **db)
        .await
        .map_err(|e| NotFound(e.to_string()))?;

    let actor_id = target.actor_id();

    let following = sqlx::query!(
        r#"
            SELECT followed_id FROM follow
            WHERE follower_id = ?
        "#,
        actor_id
    )
    .fetch_all(&mut **db)
    .await
    .unwrap();

    Ok(Json(OrderedCollection {
        ty: "OrderedCollection".to_string(),
        total_items: 1,
        ordered_items: following
            .into_iter()
            .map(|f| f.followed_id)
            .collect::<Vec<_>>(),
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
