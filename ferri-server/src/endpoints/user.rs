use main::ap;
use rocket::{State, get, http::ContentType, post, serde::json::Json};
use rocket_db_pools::Connection;

use crate::{
    Db,
    http::HttpClient,
    types::{OrderedCollection, Person, UserKey, activity, content},
};

use rocket::serde::json::serde_json;

use super::activity_type;

#[get("/users/<user>/inbox")]
pub async fn inbox(user: String) -> Json<OrderedCollection> {
    Json(OrderedCollection {
        ty: "OrderedCollection".to_string(),
        summary: format!("Inbox for {}", user),
        total_items: 0,
        ordered_items: vec![],
    })
}

#[post("/users/<user>/inbox", data = "<body>")]
pub async fn post_inbox(
    mut db: Connection<Db>,
    http: &State<HttpClient>,
    user: String,
    body: String,
) {
    let min = serde_json::from_str::<activity::MinimalActivity>(&body).unwrap();
    match min.ty.as_str() {
        "Delete" => {
            let activity = serde_json::from_str::<activity::DeleteActivity>(&body);
            dbg!(activity.unwrap());
        }
        "Follow" => {
            let activity = serde_json::from_str::<activity::FollowActivity>(&body).unwrap();
            dbg!(&activity);

            let user = http
                .get(&activity.actor)
                .activity()
                .send()
                .await
                .unwrap()
                .json::<Person>()
                .await
                .unwrap();

            sqlx::query!(
                r#"
              INSERT INTO actor (id, inbox, outbox)
              VALUES ( ?1, ?2, ?3 )
              ON CONFLICT(id) DO NOTHING;
            "#,
                activity.actor,
                user.inbox,
                user.outbox
            )
            .execute(&mut **db)
            .await
            .unwrap();

            sqlx::query!(
                r#"
              INSERT INTO follow (id, follower_id, followed_id)
              VALUES ( ?1, ?2, ?3 )
              ON CONFLICT(id) DO NOTHING;
            "#,
                activity.id,
                activity.actor,
                activity.object
            )
            .execute(&mut **db)
            .await
            .unwrap();

            let accept = activity::AcceptActivity {
                ty: "Accept".to_string(),
                actor: "https://ferri.amy.mov/users/amy".to_string(),
                object: activity.id,
            };

            let key_id = "https://ferri.amy.mov/users/amy#main-key";
            let accept_res = http
                .post(user.inbox)
                .json(&accept)
                .sign(key_id)
                .activity()
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();

            dbg!(accept_res);
        }
        unknown => {
            eprintln!("WARN: Unknown activity '{}' - {}", unknown, body);
        }
    }

    dbg!(min);
    println!("Body in inbox: {}", body);
}

#[get("/users/<user>/outbox")]
pub async fn outbox(user: String) -> Json<OrderedCollection> {
    dbg!(&user);
    Json(OrderedCollection {
        ty: "OrderedCollection".to_string(),
        summary: format!("Outbox for {}", user),
        total_items: 0,
        ordered_items: vec![],
    })
}

#[get("/users/<user>/followers")]
pub async fn followers(mut db: Connection<Db>, user: String) -> Json<OrderedCollection> {
    let target = ap::User::from_username(&user, &mut **db).await;
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
        summary: format!("Followers for {}", user),
        total_items: 1,
        ordered_items: followers
            .into_iter()
            .map(|f| f.follower_id)
            .collect::<Vec<_>>(),
    })
}

#[get("/users/<user>/following")]
pub async fn following(mut db: Connection<Db>, user: String) -> Json<OrderedCollection> {
    let target = ap::User::from_username(&user, &mut **db).await;
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
        summary: format!("Following for {}", user),
        total_items: 1,
        ordered_items: following
            .into_iter()
            .map(|f| f.followed_id)
            .collect::<Vec<_>>(),
    })
}

#[get("/users/<user>/posts/<post>")]
pub async fn post(user: String, post: String) -> (ContentType, Json<content::Post>) {
    (
        activity_type(),
        Json(content::Post {
            id: format!("https://ferri.amy.mov/users/{}/posts/{}", user, post),
            context: "https://www.w3.org/ns/activitystreams".to_string(),
            ty: "Note".to_string(),
            content: "My first post".to_string(),
            ts: "2025-04-10T10:48:11Z".to_string(),
            to: vec!["https://ferri.amy.mov/users/amy/followers".to_string()],
            cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        }),
    )
}

#[get("/users/<user>")]
pub async fn user(user: String) -> (ContentType, Json<Person>) {
    (
        activity_type(),
        Json(Person {
            context: "https://www.w3.org/ns/activitystreams".to_string(),
            ty: "Person".to_string(),
            id: format!("https://ferri.amy.mov/users/{}", user),
            name: user.clone(),
            preferred_username: user.clone(),
            followers: format!("https://ferri.amy.mov/users/{}/followers", user),
            following: format!("https://ferri.amy.mov/users/{}/following", user),
            summary: format!("ferri {}", user),
            inbox: format!("https://ferri.amy.mov/users/{}/inbox", user),
            outbox: format!("https://ferri.amy.mov/users/{}/outbox", user),
            public_key: Some(UserKey {
                id: format!("https://ferri.amy.mov/users/{}#main-key", user),
                owner: format!("https://ferri.amy.mov/users/{}", user),
                public_key: include_str!("../../../public.pem").to_string(),
            }),
        }),
    )
}
