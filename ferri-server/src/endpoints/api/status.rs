use chrono::Local;
use main::ap::{self, http::HttpClient};
use rocket::{
    FromForm, State,
    form::Form,
    post,
    serde::{Deserialize, Serialize},
};
use rocket_db_pools::Connection;
use uuid::Uuid;

use crate::{AuthenticatedUser, Db, types::content};
#[derive(Serialize, Deserialize, Debug, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct Status {
    status: String,
}

#[post("/statuses", data = "<status>")]
pub async fn new_status(
    mut db: Connection<Db>,
    http: &State<HttpClient>,
    status: Form<Status>,
    user: AuthenticatedUser,
) {
    let user = ap::User::from_actor_id(&user.actor_id, &mut **db).await;
    let outbox = ap::Outbox::for_user(user.clone(), http);

    let post_id = Uuid::new_v4();

    let uri = format!(
        "https://ferri.amy.mov/users/{}/posts/{}",
        user.username(),
        post_id
    );
    let id = user.id();
    let now = Local::now().to_rfc3339();

    let post = sqlx::query!(
        r#"
            INSERT INTO post (id, user_id, content, created_at)
            VALUES (?1, ?2, ?3, ?4)
            RETURNING *
        "#,
        uri,
        id,
        status.status,
        now
    )
    .fetch_one(&mut **db)
    .await
    .unwrap();

    let actors = sqlx::query!("SELECT * FROM actor")
        .fetch_all(&mut **db)
        .await
        .unwrap();

    for record in actors {
        // Don't send to ourselves
        if &record.id == user.actor_id() {
            continue
        }

        let create_id = format!("https://ferri.amy.mov/activities/{}", Uuid::new_v4());

        let activity = ap::Activity {
            id: create_id,
            ty: ap::ActivityType::Create,
            object: content::Post {
                context: "https://www.w3.org/ns/activitystreams".to_string(),
                id: uri.clone(),
                content: status.status.clone(),
                ty: "Note".to_string(),
                ts: Local::now().to_rfc3339(),
                to: vec![format!(
                    "https://ferri.amy.mov/users/{}/followers",
                    user.username()
                )],
                cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
            },
            to: vec![format!(
                "https://ferri.amy.mov/users/{}/followers",
                user.username()
            )],
            cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
            ..Default::default()
        };

        let actor = ap::Actor::from_raw(
            record.id.clone(),
            record.inbox.clone(),
            record.outbox.clone(),
        );
        let req = ap::OutgoingActivity {
            req: activity,
            signed_by: format!("https://ferri.amy.mov/users/{}#main-key", user.username()),
            to: actor,
        };

        req.save(&mut **db).await;
        outbox.post(req).await;
    }
}
