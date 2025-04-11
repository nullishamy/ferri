use main::ap;
use rocket::{State, post};
use rocket_db_pools::Connection;
use rocket::serde::json::serde_json;
use sqlx::Sqlite;
use url::Url;
use uuid::Uuid;
use chrono::Local;

use crate::{
    Db,
    http::HttpClient,
    types::{Person, activity},
};

fn handle_delete_activity(activity: activity::DeleteActivity) {
    dbg!(activity);
}

async fn create_actor(user: &Person, actor: String, conn: impl sqlx::Executor<'_, Database = Sqlite>) {
    sqlx::query!(
        r#"
            INSERT INTO actor (id, inbox, outbox)
            VALUES ( ?1, ?2, ?3 )
            ON CONFLICT(id) DO NOTHING;
        "#,
        actor,
        user.inbox,
        user.outbox
    )
    .execute(conn)
    .await
    .unwrap();
}

async fn create_user(user: &Person, actor: String, conn: impl sqlx::Executor<'_, Database = Sqlite>) {
    // HACK: Allow us to formulate a `user@host` username by assuming the actor is on the same host as the user
    let url = Url::parse(&actor).unwrap();
    let host = url.host_str().unwrap();
    let username = format!("{}@{}", user.name, host);

    let uuid = Uuid::new_v4().to_string();
    sqlx::query!(
        r#"
            INSERT INTO user (id, username, actor_id, display_name)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(actor_id) DO NOTHING;
        "#,
        uuid,
        username,
        actor,
        user.preferred_username
    )
    .execute(conn)
    .await
    .unwrap();
}

async fn create_follow(activity: &activity::FollowActivity, conn: impl sqlx::Executor<'_, Database = Sqlite>) {
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
    .execute(conn)
    .await
    .unwrap();
}

async fn handle_follow_activity(followed_account: String, activity: activity::FollowActivity, http: &HttpClient, mut db: Connection<Db>) {
    let user = http
        .get(&activity.actor)
        .activity()
        .send()
        .await
        .unwrap()
        .json::<Person>()
        .await
        .unwrap();

    create_actor(&user, activity.actor.clone(), &mut **db).await;
    create_user(&user, activity.actor.clone(), &mut **db).await;
    create_follow(&activity, &mut **db).await;

    let follower = ap::User::from_actor_id(&activity.actor, &mut **db).await;
    let followed = ap::User::from_username(&followed_account, &mut **db).await;
    let outbox = ap::Outbox::for_user(followed.clone(), http);

    let activity = ap::Activity {
        id: format!("https://ferri.amy.mov/activities/{}", Uuid::new_v4()),
        ty: ap::ActivityType::Accept,
        object: activity.id,
        ..Default::default()
    };

    let req = ap::OutgoingActivity {
        signed_by: format!(
            "https://ferri.amy.mov/users/{}#main-key",
            followed.username()
        ),
        req: activity,
        to: follower.actor().clone(),
    };

    req.save(&mut **db).await;
    outbox.post(req).await;
}

async fn handle_create_activity(activity: activity::CreateActivity,http: &HttpClient, mut db: Connection<Db>) {
    assert!(&activity.object.ty == "Note");
    let user = http
        .get(&activity.actor)
        .activity()
        .send()
        .await
        .unwrap()
        .json::<Person>()
        .await
        .unwrap();

    create_actor(&user, activity.actor.clone(), &mut **db).await;
    create_user(&user, activity.actor.clone(), &mut **db).await;

    let user = ap::User::from_actor_id(&activity.actor, &mut **db).await;

    let post_id = Uuid::new_v4();

    let uri = format!(
        "https://ferri.amy.mov/users/{}/posts/{}",
        user.username(),
        post_id
    );
    let id = user.id();
    let now = Local::now().to_rfc3339();
    let content = activity.object.content.clone();

    sqlx::query!(r#"
        INSERT INTO post (id, user_id, content, created_at)
        VALUES (?1, ?2, ?3, ?4)
    "#, uri, id, content, now)
        .execute(&mut **db)
        .await.unwrap();
}

#[post("/users/<user>/inbox", data = "<body>")]
pub async fn inbox(db: Connection<Db>, http: &State<HttpClient>, user: String, body: String) {
    let min = serde_json::from_str::<activity::MinimalActivity>(&body).unwrap();
    match min.ty.as_str() {
        "Delete" => {
            let activity = serde_json::from_str::<activity::DeleteActivity>(&body).unwrap();
            handle_delete_activity(activity);
        }
        "Follow" => {
            let activity = serde_json::from_str::<activity::FollowActivity>(&body).unwrap();
            handle_follow_activity(user, activity, http.inner(), db).await;
        },
        "Create" => {
            let activity = serde_json::from_str::<activity::CreateActivity>(&body).unwrap();
            handle_create_activity(activity, http.inner(), db).await;
        },
        unknown => {
            eprintln!("WARN: Unknown activity '{}' - {}", unknown, body);
        }
    }

    dbg!(min);
    println!("Body in inbox: {}", body);
}
