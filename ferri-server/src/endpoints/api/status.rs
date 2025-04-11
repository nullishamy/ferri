use rocket::{
    FromForm,
    form::Form,
    post,
    serde::{Deserialize, Serialize},
};
use rocket_db_pools::Connection;
use uuid::Uuid;
use main::ap;

use crate::{AuthenticatedUser, Db};
#[derive(Serialize, Deserialize, Debug, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct Status {
    status: String,
}

#[post("/statuses", data = "<status>")]
pub async fn new_status(mut db: Connection<Db>, status: Form<Status>, user: AuthenticatedUser) {
    let user = ap::User::from_actor_id(&user.actor_id, &mut **db).await;
    let post_id = Uuid::new_v4();
    let uri = format!("https://ferri.amy.mov/users/{}/posts/{}", user.username(), post_id);
    let id = user.id();

    let post = sqlx::query!(
        r#"
            INSERT INTO post (id, user_id, content)
            VALUES (?1, ?2, ?3)
            RETURNING *
        "#,
        uri,
        id,
        status.status
    )
    .fetch_one(&mut **db)
    .await
    .unwrap();

    dbg!(user, status, post);
}
