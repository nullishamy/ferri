use chrono::Local;
use main::ap;
use rocket::serde::json::serde_json;
use rocket::{State, post};
use rocket_db_pools::Connection;
use sqlx::Sqlite;
use url::Url;
use uuid::Uuid;
use tracing::{event, span, Level, debug, warn, info};

use crate::{
    Db,
    http::HttpClient,
    types::{Person, content::Post, activity},
};

fn handle_delete_activity(activity: activity::DeleteActivity) {
    warn!(?activity, "unimplemented delete activity");
}

async fn create_actor(
    user: &Person,
    actor: &str,
    conn: impl sqlx::Executor<'_, Database = Sqlite>,
) {
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

async fn create_user(
    user: &Person,
    actor: &str,
    conn: impl sqlx::Executor<'_, Database = Sqlite>,
) {
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

async fn create_follow(
    activity: &activity::FollowActivity,
    conn: impl sqlx::Executor<'_, Database = Sqlite>,
) {
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

async fn handle_follow_activity(
    followed_account: &str,
    activity: activity::FollowActivity,
    http: &HttpClient,
    mut db: Connection<Db>,
) {
    let user = http
        .get(&activity.actor)
        .activity()
        .send()
        .await
        .unwrap()
        .json::<Person>()
        .await
        .unwrap();

    create_actor(&user, &activity.actor, &mut **db).await;
    create_user(&user, &activity.actor, &mut **db).await;
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

async fn handle_like_activity(activity: activity::LikeActivity, mut db: Connection<Db>) {
    warn!(?activity, "unimplemented like activity");
    
    let target_post = sqlx::query!("SELECT * FROM post WHERE uri = ?1", activity.object)
        .fetch_one(&mut **db)
        .await;

    if let Ok(post) = target_post {
        warn!(?post, "tried to like post");
    } else {
        warn!(post = ?activity.object, "could not find post");
    }
}

async fn handle_boost_activity(
    activity: activity::BoostActivity,
    http: &HttpClient,
    mut db: Connection<Db>,
) {
    let key_id = "https://ferri.amy.mov/users/amy#main-key";
    dbg!(&activity);
    let post = http
        .get(&activity.object)
        .activity()
        .sign(&key_id)
        .send()
        .await
        .unwrap()
        .json::<Post>()
        .await
        .unwrap();

    dbg!(&post);
    let attribution = post.attributed_to.unwrap();
    let post_user = http
        .get(&attribution)
        .activity()
        .sign(&key_id)
        .send()
        .await
        .unwrap()
        .json::<Person>()
        .await
        .unwrap();

    let user = http
        .get(&activity.actor)
        .activity()
        .sign(&key_id)
        .send()
        .await
        .unwrap()
        .json::<Person>()
        .await
        .unwrap();

        dbg!(&post_user);

    debug!("creating actor {}", activity.actor);
    create_actor(&user, &activity.actor, &mut **db).await;

    debug!("creating user {}", activity.actor);
    create_user(&user, &activity.actor, &mut **db).await;

    debug!("creating actor {}", attribution);
    create_actor(&post_user, &attribution, &mut **db).await;

    debug!("creating user {}", attribution);
    create_user(&post_user, &attribution, &mut **db).await;
    
    let attributed_user = ap::User::from_actor_id(&attribution, &mut **db).await;
    let actor_user = ap::User::from_actor_id(&activity.actor, &mut **db).await;
    
    let base_id = ap::new_id();
    let now = ap::new_ts();
    
    let reblog_id = ap::new_id();

    let attr_id = attributed_user.id();
    sqlx::query!("
       INSERT INTO post (id, uri, user_id, content, created_at)
       VALUES (?1, ?2, ?3, ?4, ?5)
    ", reblog_id, post.id, attr_id, post.content, post.ts)
        .execute(&mut **db)
        .await
        .unwrap();

    let uri = format!("https://ferri.amy.mov/users/{}/posts/{}", actor_user.id(), post.id);
    let user_id = actor_user.id();
    
    sqlx::query!("
       INSERT INTO post (id, uri, user_id, content, created_at, boosted_post_id)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6)
    ", base_id, uri, user_id, "", now, reblog_id)
        .execute(&mut **db)
        .await
        .unwrap();

}

async fn handle_create_activity(
    activity: activity::CreateActivity,
    http: &HttpClient,
    mut db: Connection<Db>,
) {
    assert!(&activity.object.ty == "Note");
    debug!("resolving user {}", activity.actor);
    
    let user = http
        .get(&activity.actor)
        .activity()
        .send()
        .await
        .unwrap()
        .json::<Person>()
        .await
        .unwrap();

    debug!("creating actor {}", activity.actor);
    create_actor(&user, &activity.actor, &mut **db).await;

    debug!("creating user {}", activity.actor);
    create_user(&user, &activity.actor, &mut **db).await;

    let user = ap::User::from_actor_id(&activity.actor, &mut **db).await;
    debug!("user created {:?}", user);

    let user_id = user.id();
    let now = Local::now().to_rfc3339();
    let content = activity.object.content.clone();
    let post_id = Uuid::new_v4().to_string();
    let uri = activity.id;

    info!(post_id, "creating post");

    sqlx::query!(
        r#"
        INSERT INTO post (id, uri, user_id, content, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
    "#,
        post_id,
        uri,
        user_id,
        content,
        now
    )
        .execute(&mut **db)
        .await
        .unwrap();
}

#[post("/users/<user>/inbox", data = "<body>")]
pub async fn inbox(db: Connection<Db>, http: &State<HttpClient>, user: &str, body: String) {
    let min = serde_json::from_str::<activity::MinimalActivity>(&body).unwrap();
    let inbox_span = span!(Level::INFO, "inbox-post", user_id = user);

    let _enter = inbox_span.enter();
    event!(Level::INFO, ?min, "received an activity");
    
    match min.ty.as_str() {
        "Delete" => {
            let activity = serde_json::from_str::<activity::DeleteActivity>(&body).unwrap();
            handle_delete_activity(activity);
        }
        "Follow" => {
            let activity = serde_json::from_str::<activity::FollowActivity>(&body).unwrap();
            handle_follow_activity(user, activity, http.inner(), db).await;
        }
        "Create" => {
            let activity = serde_json::from_str::<activity::CreateActivity>(&body).unwrap();
            handle_create_activity(activity, http.inner(), db).await;
        }
        "Like" => {
            let activity = serde_json::from_str::<activity::LikeActivity>(&body).unwrap();
            handle_like_activity(activity, db).await;
        }
        "Announce" => {
            let activity = serde_json::from_str::<activity::BoostActivity>(&body).unwrap();
            handle_boost_activity(activity, http.inner(), db).await;
        }
         
        act => {
            warn!(act, body, "unknown activity");
        }
    }

    debug!("body in inbox: {}", body);
    drop(_enter)
}
