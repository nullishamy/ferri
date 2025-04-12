use main::ap;
use rocket::{get, http::ContentType, serde::json::Json};
use rocket_db_pools::Connection;

use crate::{
    Db,
    types::{OrderedCollection, Person, UserKey, content},
};

use super::activity_type;

#[get("/users/<user>/inbox")]
pub async fn inbox(user: String) -> Json<OrderedCollection> {
    Json(OrderedCollection {
        ty: "OrderedCollection".to_string(),
        total_items: 0,
        ordered_items: vec![],
    })
}

#[get("/users/<user>/outbox")]
pub async fn outbox(user: String) -> Json<OrderedCollection> {
    dbg!(&user);
    Json(OrderedCollection {
        ty: "OrderedCollection".to_string(),
        total_items: 0,
        ordered_items: vec![],
    })
}

#[get("/users/<uuid>/followers")]
pub async fn followers(mut db: Connection<Db>, uuid: &str) -> Json<OrderedCollection> {
    let target = ap::User::from_id(uuid, &mut **db).await;
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

    Json(OrderedCollection {
        ty: "OrderedCollection".to_string(),
        total_items: 1,
        ordered_items: followers
            .into_iter()
            .map(|f| f.follower_id)
            .collect::<Vec<_>>(),
    })
}

#[get("/users/<uuid>/following")]
pub async fn following(mut db: Connection<Db>, uuid: &str) -> Json<OrderedCollection> {
    let target = ap::User::from_id(uuid, &mut **db).await;
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

    Json(OrderedCollection {
        ty: "OrderedCollection".to_string(),
        total_items: 1,
        ordered_items: following
            .into_iter()
            .map(|f| f.followed_id)
            .collect::<Vec<_>>(),
    })
}

#[get("/users/<uuid>/posts/<post>")]
pub async fn post(
    mut db: Connection<Db>,
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
            id: format!("https://ferri.amy.mov/users/{}/posts/{}", uuid, post.id),
            ty: "Note".to_string(),
            content: post.content,
            ts: post.created_at,
            to: vec!["https://ferri.amy.mov/users/amy/followers".to_string()],
            cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        }),
    )
}

#[get("/users/<uuid>")]
pub async fn user(mut db: Connection<Db>, uuid: &str) -> (ContentType, Json<Person>) {
    let user = ap::User::from_id(uuid, &mut **db).await;
    (
        activity_type(),
        Json(Person {
            context: "https://www.w3.org/ns/activitystreams".to_string(),
            ty: "Person".to_string(),
            id: user.id().to_string(),
            name: user.username().to_string(),
            preferred_username: user.display_name().to_string(),
            followers: format!("https://ferri.amy.mov/users/{}/followers", uuid),
            following: format!("https://ferri.amy.mov/users/{}/following", uuid),
            summary: format!("ferri {}", user.username()),
            inbox: format!("https://ferri.amy.mov/users/{}/inbox", uuid),
            outbox: format!("https://ferri.amy.mov/users/{}/outbox", uuid),
            public_key: Some(UserKey {
                id: format!("https://ferri.amy.mov/users/{}#main-key", uuid),
                owner: format!("https://ferri.amy.mov/users/{}", uuid),
                public_key: include_str!("../../../public.pem").to_string(),
            }),
        }),
    )
}
