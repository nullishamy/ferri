use crate::types::{DbError, ObjectUri, ObjectUuid, db};
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::SqliteConnection;
use tracing::{info, error};

const SQLITE_TIME_FMT: &str = "%Y-%m-%d %H:%M:%S";

fn parse_ts(ts: String) -> Option<DateTime<Utc>> {
    // Depending on how the TS is queried it may be naive (so get it back to utc)
    // or it may have a timezone associated with it
    let dt = NaiveDateTime::parse_from_str(&ts, SQLITE_TIME_FMT)
        .map(|ndt| {
            ndt.and_utc()
        })
        .or_else(|_| {
            DateTime::parse_from_rfc3339(&ts)
                .map(|dt| dt.to_utc())
        });
    
    if let Err(err) = dt {
        error!("could not parse datetime {} ({}), db weirdness", ts, err);
        return None
    }
        
    Some(dt.unwrap())
}

pub async fn user_by_id(id: ObjectUuid, conn: &mut SqliteConnection) -> Result<db::User, DbError> {
    info!("fetching user by uuid '{:?}' from the database", id);

    let record = sqlx::query!(
        r#"
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
        u.created_at,
        u.icon_url
      FROM "user" u 
      INNER JOIN "actor" a ON u.actor_id = a.id
      WHERE u.id = ?1
    "#,
        id.0
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| DbError::FetchError(e.to_string()))?;

    let follower_count = sqlx::query_scalar!(
        r#"
      SELECT COUNT(follower_id)
      FROM "follow"
      WHERE followed_id = ?1
    "#,
        record.actor_id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| DbError::FetchError(e.to_string()))?;

    let last_post_at = sqlx::query_scalar!(
        r#"
      SELECT datetime(p.created_at)
      FROM post p 
      WHERE p.user_id = ?1
      ORDER BY datetime(p.created_at) DESC 
      LIMIT 1
    "#,
        record.user_id
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| DbError::FetchError(e.to_string()))?
    .flatten()
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
            outbox: record.outbox,
        },
        acct: record.acct,
        remote: record.remote,
        username: record.username,
        display_name: record.display_name,
        created_at: user_created,
        url: record.url,
        posts: db::UserPosts { last_post_at },
        icon_url: record.icon_url
    })
}

pub async fn user_by_actor_uri(uri: ObjectUri, conn: &mut SqliteConnection) -> Result<db::User, DbError> {
    info!("fetching user by actor_uri '{:?}' from the database", uri);

    let record = sqlx::query!(
        r#"
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
        u.created_at,
        u.icon_url
      FROM "user" u 
      INNER JOIN "actor" a ON u.actor_id = a.id
      WHERE u.actor_id = ?1
    "#,
        uri.0
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| DbError::FetchError(e.to_string()))?;

    let follower_count = sqlx::query_scalar!(
        r#"
      SELECT COUNT(follower_id)
      FROM "follow"
      WHERE followed_id = ?1
    "#,
        record.actor_id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| DbError::FetchError(e.to_string()))?;

    let last_post_at = sqlx::query_scalar!(
        r#"
      SELECT datetime(p.created_at)
      FROM post p 
      WHERE p.user_id = ?1
      ORDER BY datetime(p.created_at) DESC 
      LIMIT 1
    "#,
        record.user_id
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| DbError::FetchError(e.to_string()))?
    .flatten()
    .and_then(|ts| {
        info!("parsing timestamp {}", ts);
        parse_ts(ts)
    });

    let user_created = parse_ts(record.created_at).expect("no db corruption");

    info!("user {:?} has {} followers", record.user_id, follower_count);
    info!("user {:?} last posted {:?}", record.user_id, last_post_at);

    Ok(db::User {
        id: ObjectUuid(record.user_id),
        actor: db::Actor {
            id: ObjectUri(record.actor_id),
            inbox: record.inbox,
            outbox: record.outbox,
        },
        acct: record.acct,
        remote: record.remote,
        username: record.username,
        display_name: record.display_name,
        created_at: user_created,
        url: record.url,
        posts: db::UserPosts { last_post_at },
        icon_url: record.icon_url
    })
}

pub async fn posts_for_user_id(
    id: ObjectUuid,
    conn: &mut SqliteConnection
) -> Result<Vec<db::Post>, DbError> {
    let mut out = vec![];
    let posts = sqlx::query!(r#"
      SELECT
        p.id as "post_id", u.id as "user_id",
        p.content, p.uri as "post_uri", u.username, u.display_name,
        u.actor_id, p.created_at as "post_created",
        p.boosted_post_id, a.inbox, a.outbox, u.created_at as "user_created",
        u.acct, u.remote, u.url as "user_url", u.icon_url
      FROM post p
      INNER JOIN user u on p.user_id = u.id
      INNER JOIN actor a ON u.actor_id = a.id
      WHERE p.user_id = ?
    "#, id.0)
        .fetch_all(&mut *conn)
        .await
        .unwrap();
    
    for record in posts {
        let user_created = parse_ts(record.user_created)
            .expect("no db corruption");
        
        out.push(db::Post {
            id: ObjectUuid(record.post_id),
            uri: ObjectUri(record.post_uri),
            user: db::User {
                id: ObjectUuid(record.user_id),
                actor: db::Actor {
                    id: ObjectUri(record.actor_id),
                    inbox: record.inbox,
                    outbox: record.outbox,
                },
                acct: record.acct,
                remote: record.remote,
                username: record.username,
                display_name: record.display_name,
                created_at: user_created,
                url: record.user_url,
                icon_url: record.icon_url,
                posts: db::UserPosts {
                    last_post_at: None
                }
            },
            content: record.content,
            created_at: parse_ts(record.post_created).unwrap(),
            boosted_post: None
        })
    }

    Ok(out)
}
