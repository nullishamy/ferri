use crate::types::{DbError, db};
use sqlx::SqliteConnection;

pub async fn new_user(user: db::User, conn: &mut SqliteConnection) -> Result<db::User, DbError> {
    let ts = user.created_at.to_rfc3339();
    sqlx::query!(
        r#"
      INSERT INTO user (id, acct, url, created_at, remote,
                        username, actor_id, display_name, icon_url)
      VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
      ON CONFLICT(actor_id) DO NOTHING
    "#,
        user.id.0,
        user.acct,
        user.url,
        ts,
        user.remote,
        user.username,
        user.actor.id.0,
        user.display_name,
        user.icon_url
    )
    .execute(conn)
    .await
    .map_err(|e| DbError::CreationError(e.to_string()))?;

    Ok(user)
}

pub async fn new_actor(
    actor: db::Actor,
    conn: &mut SqliteConnection,
) -> Result<db::Actor, DbError> {
    sqlx::query!(
        r#"
      INSERT INTO actor (id, inbox, outbox)
      VALUES (?1, ?2, ?3)
      ON CONFLICT(id) DO NOTHING
    "#,
        actor.id.0,
        actor.inbox,
        actor.outbox
    )
    .execute(conn)
    .await
    .map_err(|e| DbError::CreationError(e.to_string()))?;

    Ok(actor)
}

pub async fn new_follow(
    follow: db::Follow,
    conn: &mut SqliteConnection,
) -> Result<db::Follow, DbError> {
    sqlx::query!(
        r#"
      INSERT INTO follow (id, follower_id, followed_id)
      VALUES (?1, ?2, ?3)
    "#,
        follow.id.0,
        follow.follower.0,
        follow.follower.0,
    )
    .execute(conn)
    .await
    .map_err(|e| DbError::CreationError(e.to_string()))?;

    Ok(follow)
}

pub async fn new_attachment(
    attachment: db::Attachment,
    conn: &mut SqliteConnection
) -> Result<db::Attachment, DbError> {
    sqlx::query!(
        r#"
      INSERT INTO attachment (id, post_id, url, media_type, marked_sensitive, alt)
      VALUES (?1, ?2, ?3, ?4, ?5, ?6)
    "#,
        attachment.id.0,
        attachment.post_id.0,
        attachment.url,
        attachment.media_type,
        attachment.sensitive,
        attachment.alt
    )
    .execute(conn)
    .await
    .map_err(|e| DbError::CreationError(e.to_string()))?;

    Ok(attachment)
}

pub async fn new_post(
    post: db::Post,
    conn: &mut SqliteConnection,
) -> Result<db::Post, DbError> {
    let ts = post.created_at.to_rfc3339();
    let boosted = post.boosted_post.as_ref().map(|b| &b.id.0);
    
    sqlx::query!(
        r#"
      INSERT INTO post (id, uri, user_id, content, created_at, boosted_post_id)
      VALUES (?1, ?2, ?3, ?4, ?5, ?6)
      ON CONFLICT(uri) DO NOTHING
    "#,
        post.id.0,
        post.uri.0,
        post.user.id.0,
        post.content,
        ts,
        boosted
    )
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::CreationError(e.to_string()))?;

    for attachment in post.attachments.clone() {
        new_attachment(attachment, &mut *conn).await?;
    }

    Ok(post)
}


