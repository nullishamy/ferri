use crate::types_rewrite::db;
use sqlx::SqliteConnection;
use crate::types_rewrite::DbError;

pub async fn new_user(user: db::User, conn: &mut SqliteConnection) -> Result<db::User, DbError> {
    let ts = user.created_at.to_rfc3339();
    sqlx::query!(r#"
      INSERT INTO user (id, acct, url, created_at, remote, username, actor_id, display_name)
      VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
    "#, user.id.0, user.acct, user.url, ts,
        user.remote, user.username, user.actor.id.0, user.display_name
    )
        .execute(conn)
        .await
        .map_err(|e| DbError::CreationError(e.to_string()))?;

    Ok(user)
}

pub async fn new_actor(actor: db::Actor, conn: &mut SqliteConnection) -> Result<db::Actor, DbError> {
    sqlx::query!(r#"
      INSERT INTO actor (id, inbox, outbox)
      VALUES (?1, ?2, ?3)
    "#, actor.id.0, actor.inbox, actor.outbox)
        .execute(conn)
        .await
        .map_err(|e| DbError::CreationError(e.to_string()))?;

    Ok(actor)
}
