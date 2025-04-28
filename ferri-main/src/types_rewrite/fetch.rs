use crate::types_rewrite::{ObjectUuid, ObjectUri, db};
use sqlx::SqliteConnection;
use thiserror::Error;
use tracing::info;
use chrono::{NaiveDateTime, DateTime, Utc};

const SQLITE_TIME_FMT: &'static str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("an unknown error occured when fetching: {0}")]
    Unknown(String)
}

fn parse_ts(ts: String) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(&ts, SQLITE_TIME_FMT)
        .ok()
        .map(|nt| nt.and_utc())
}

pub async fn user_by_id(id: ObjectUuid, conn: &mut SqliteConnection) -> Result<db::User, FetchError> {
    info!("fetching user by uuid '{:?}' from the database", id);
    
    let record = sqlx::query!(r#"
      SELECT
        u.id as "user_id",
        u.username,
        u.actor_id,
        u.display_name,
        a.inbox,
        a.outbox,
        u.url,
        u.acct,
        u.remote,
        u.created_at
      FROM "user" u 
      INNER JOIN "actor" a ON u.actor_id = a.id
      WHERE u.id = ?1
    "#, id.0)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| FetchError::Unknown(e.to_string()))?;

    let follower_count = sqlx::query_scalar!(r#"
      SELECT COUNT(follower_id)
      FROM "follow"
      WHERE followed_id = ?1
    "#, record.actor_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| FetchError::Unknown(e.to_string()))?;

    let last_post_at = sqlx::query_scalar!(r#"
      SELECT datetime(p.created_at)
      FROM post p 
      WHERE p.user_id = ?1
      ORDER BY datetime(p.created_at) DESC 
      LIMIT 1
    "#, record.user_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| FetchError::Unknown(e.to_string()))?
        .and_then(|ts| {
            info!("parsing timestamp {}", ts);
            parse_ts(ts)
        });

    let user_created = parse_ts(record.created_at).expect("no db corruption");

    info!("user {:?} has {} followers", id, follower_count);
    info!("user {:?} last posted {:?}", id, last_post_at);
    
    Ok(db::User {
        id: ObjectUuid(record.user_id),
        actor: db::Actor {
            id: ObjectUri(record.actor_id),
            inbox: record.inbox,
            outbox: record.outbox
        },
        acct: record.acct,
        remote: record.remote,
        username: record.username,
        display_name: record.display_name,
        created_at: user_created,
        url: record.url,
        posts: db::UserPosts {
            last_post_at
        }
    })
 }
