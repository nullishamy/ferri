use crate::{AuthenticatedUser, Db};
use main::types::{api, get};
use rocket::{
    get,
    serde::json::Json,
};
use rocket_db_pools::Connection;

#[get("/timelines/home")]
pub async fn home(
    mut db: Connection<Db>,
    user: AuthenticatedUser,
) -> Json<Vec<api::Status>> {
    let posts = get::home_timeline(user.actor_id, &mut **db)
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.into())
        .collect();

    Json(posts)
}
