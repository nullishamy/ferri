use main::ap;
use rocket::{get, http::ContentType, serde::json::Json, State};
use rocket_db_pools::Connection;
use rocket::response::status::NotFound;

use crate::{
    Config,
    Db,
    types::{OrderedCollection, Person, UserKey, content},
};

use super::activity_type;

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
pub async fn followers(mut db: Connection<Db>, uuid: &str) -> Result<Json<OrderedCollection>, NotFound<String>> {
    let target = ap::User::from_id(uuid, &mut **db)
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
pub async fn following(mut db: Connection<Db>, uuid: &str) -> Result<Json<OrderedCollection>, NotFound<String>> {
    let target = ap::User::from_id(uuid, &mut **db)
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
    config: &State<Config>,
    uuid: &str,
    post: String,
) -> (ContentType, Json<content::Post>) {
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
        Json(content::Post {
            context: "https://www.w3.org/ns/activitystreams".to_string(),
            id: config.post_url(uuid, &post.id),
            attributed_to: Some(config.user_url(uuid)),
            ty: "Note".to_string(),
            content: post.content,
            ts: post.created_at,
            to: vec![config.followers_url(uuid)],
            cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        }),
    )
}

#[get("/users/<uuid>")]
pub async fn user(
    mut db: Connection<Db>,
    config: &State<Config>,
    uuid: &str
) -> Result<(ContentType, Json<Person>), NotFound<String>> {
    let user = ap::User::from_id(uuid, &mut **db)
        .await
        .map_err(|e| NotFound(e.to_string()))?;

    Ok((
        activity_type(),
        Json(Person {
            context: "https://www.w3.org/ns/activitystreams".to_string(),
            ty: "Person".to_string(),
            id: config.user_url(user.id()),
            name: user.username().to_string(),
            preferred_username: user.display_name().to_string(),
            followers: config.followers_url(user.id()),
            following: config.following_url(user.id()),
            summary: format!("ferri {}", user.username()),
            inbox: config.inbox_url(user.id()),
            outbox: config.outbox_url(user.id()),
            public_key: Some(UserKey {
                id: format!("https://ferri.amy.mov/users/{}#main-key", uuid),
                owner: config.user_url(user.id()),
                public_key: include_str!("../../../public.pem").to_string(),
            }),
        }),
    ))
}
