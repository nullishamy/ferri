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

pub async fn user_by_id(
    id: ObjectUuid,
    conn: &mut SqliteConnection
) -> Result<db::User, DbError> {
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
        id: ObjectUuid(record.user_id.clone()),
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
        icon_url: record.icon_url,
        key_id: format!(
            "https://ferri.amy.mov/users/{}#main-key",
            record.user_id
        )
    })
}

pub async fn user_by_username(
    username: &str,
    conn: &mut SqliteConnection
) -> Result<db::User, DbError> {
    info!("fetching user by username '{}' from the database", username);

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
      WHERE u.username = ?1
    "#,
        username
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

    info!("user {} has {} followers", record.user_id, follower_count);
    info!("user {} last posted {:?}", record.user_id, last_post_at);

    Ok(db::User {
        id: ObjectUuid(record.user_id.clone()),
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
        icon_url: record.icon_url,
        key_id: format!(
            "https://ferri.amy.mov/users/{}#main-key",
            record.user_id
        )
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
        id: ObjectUuid(record.user_id.clone()),
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
        icon_url: record.icon_url,
        key_id: format!(
            "https://ferri.amy.mov/users/{}#main-key",
            record.user_id
        )
    })
}

pub async fn attachments_for_post(
    post_id: ObjectUuid,
    conn: &mut SqliteConnection
)-> Result<Vec<db::Attachment>, DbError> {
    let attachments = sqlx::query!(
        "SELECT * FROM attachment WHERE post_id = ?",
        post_id.0
    )
        .fetch_all(&mut *conn)
        .await
        .unwrap();

    let attachments = attachments.into_iter()
        .map(|at| {
            db::Attachment {
                id: ObjectUuid(at.id),
                post_id: ObjectUuid(at.post_id),
                url: at.url,
                media_type: Some(at.media_type),
                sensitive: at.marked_sensitive,
                alt: at.alt
            }
        })
        .collect::<Vec<_>>();
    
    Ok(attachments)
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
        let attachments = attachments_for_post(ObjectUuid(record.post_id.clone()), conn)
            .await
            .unwrap();
        
        let user_created = parse_ts(record.user_created)
            .expect("no db corruption");
        
        out.push(db::Post {
            id: ObjectUuid(record.post_id),
            uri: ObjectUri(record.post_uri),
            user: db::User {
                id: ObjectUuid(record.user_id.clone()),
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
                },
                key_id: format!(
                    "https://ferri.amy.mov/users/{}#main-key",
                    record.user_id
                )
            },
            attachments,
            content: record.content,
            created_at: parse_ts(record.post_created).unwrap(),
            boosted_post: None
        })
    }

    Ok(out)
}

pub async fn home_timeline(
    actor: ObjectUri,
    conn: &mut SqliteConnection
) -> Result<Vec<db::Post>, DbError> {
    #[derive(sqlx::FromRow, Debug, Clone)]
    struct Post {
        is_boost_source: bool,
        post_id: String,
        user_id: String,
        post_uri: String,
        content: String,
        post_created: String,
        user_created: String,
        actor_id: String,
        acct: String,
        remote: bool,
        boosted_post_id: Option<String>,
        display_name: String,
        username: String,
        icon_url: String,
        user_url: String,
        inbox: String,
        outbox: String
    }

    fn make_into_db(p: Post, attachments: Vec<db::Attachment>) -> db::Post {
        db::Post {
            id: ObjectUuid(p.post_id),
            uri: ObjectUri(p.post_uri),
            user: db::User {
                id: ObjectUuid(p.user_id.clone()),
                actor: db::Actor {
                    id: ObjectUri(p.actor_id),
                    inbox: p.inbox,
                    outbox: p.outbox
                },
                username: p.username,
                display_name: p.display_name,
                acct: p.acct,
                remote: p.remote,
                url: p.user_url,
                created_at: parse_ts(p.user_created).unwrap(),
                icon_url: p.icon_url,
                posts: db::UserPosts {
                    last_post_at: None
                },
                key_id: format!(
                    "https://ferri.amy.mov/users/{}#main-key",
                    p.user_id
                )
            },
            content: p.content,
            created_at: parse_ts(p.post_created).unwrap(),
            boosted_post: None,
            attachments
        }
    }

    // FIXME: query! can't cope with this. returns a type error
    let posts = sqlx::query_as::<_, Post>(
        r#"   
            WITH RECURSIVE get_home_timeline_with_boosts(
              id, boosted_post_id, is_boost_source
            ) AS
            (
              SELECT p.id, p.boosted_post_id, 0 as is_boost_source
              FROM post p
              WHERE p.user_id IN (
                SELECT u.id
                FROM follow f 
                INNER JOIN user u ON u.actor_id = f.followed_id 
                WHERE f.follower_id = $1
              )
            UNION
              SELECT p.id, p.boosted_post_id, 1 as is_boost_source
              FROM post p
              JOIN get_home_timeline_with_boosts tl ON tl.boosted_post_id = p.id
           )
           SELECT is_boost_source, p.id as "post_id", u.id as "user_id",
                  p.content, p.uri as "post_uri", u.username, u.display_name,
                  u.actor_id, p.created_at as "post_created", p.boosted_post_id, u.icon_url, u.url as "user_url",
                  a.inbox, a.outbox, u.acct, u.remote, u.created_at as "user_created"
           FROM get_home_timeline_with_boosts
           JOIN post p ON p.id = get_home_timeline_with_boosts.id
           JOIN actor a ON u.actor_id = a.id
           JOIN user u ON u.id = p.user_id;
        "#,
    )
        .bind(actor.0)
        .fetch_all(&mut *conn)
        .await
        .unwrap();

    let mut out = vec![];
    for post in posts.iter() {
        let boost_id = post.boosted_post_id.clone();
        let is_boost_base = post.is_boost_source;
        let attachments = attachments_for_post(ObjectUuid(post.post_id.clone()), &mut *conn)
            .await
            .unwrap();
        
        let mut base = make_into_db(post.clone(), attachments);
        if let Some(boost_id) = boost_id {

            let attachments = attachments_for_post(ObjectUuid(boost_id.clone()), &mut *conn)
                .await
                .unwrap();
            
            let boost = posts.iter().find(|p| p.post_id == boost_id).unwrap();
            let boost = make_into_db(boost.clone(), attachments);
            base.boosted_post = Some(Box::new(boost));
        }

        if !is_boost_base {
            out.push(base);
        }
    }

    Ok(out)
}

pub async fn followers_for_user(
    user_id: ObjectUuid,
    conn: &mut SqliteConnection
) -> Result<Vec<db::Follow>, DbError> {
    let followers = sqlx::query!(
        "SELECT * FROM follow WHERE followed_id = ?",
        user_id.0
    )
        .fetch_all(&mut *conn)
        .await
        .unwrap();

    let followers = followers.into_iter()
        .map(|f| {
            db::Follow {
                id: ObjectUri(f.id),
                follower: ObjectUri(f.follower_id),
                followed: ObjectUri(f.followed_id)
            }
        })
        .collect::<Vec<_>>();
    
    Ok(followers)   
}

pub async fn following_for_user(
    user_id: ObjectUuid,
    conn: &mut SqliteConnection
) -> Result<Vec<db::Follow>, DbError> {
    let followers = sqlx::query!(
        "SELECT * FROM follow WHERE follower_id = ?",
        user_id.0
    )
        .fetch_all(&mut *conn)
        .await
        .unwrap();

    let followers = followers.into_iter()
        .map(|f| {
            db::Follow {
                id: ObjectUri(f.id),
                follower: ObjectUri(f.follower_id),
                followed: ObjectUri(f.followed_id)
            }
        })
        .collect::<Vec<_>>();
    
    Ok(followers)   
}
