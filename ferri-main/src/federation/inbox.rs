use crate::types::{ap, as_context, db, get, make, Object, ObjectUri, ObjectUuid};
use crate::ap::http::HttpClient;

use super::http::HttpWrapper;
use super::outbox::OutboxRequest;
use super::QueueMessage;

use chrono::DateTime;
use tracing::warn;

#[derive(Debug)]
pub enum InboxRequest {
    Delete(ap::DeleteActivity, db::User),
    Follow {
        activity: ap::FollowActivity,
        followed: db::User,
        conn: sqlx::SqliteConnection,
        outbound: super::QueueHandle
    },
    Create(ap::CreateActivity, db::User, sqlx::SqliteConnection),
    Like(ap::LikeActivity, db::User),
    Boost(ap::BoostActivity, db::User)
}

fn key_id(user: &db::User) -> String {
    format!("https://ferri.amy.mov/users/{}#main-key", user.id.0)
}

pub async fn handle_inbox_request(
    req: InboxRequest,
    http: &HttpClient,
) {
    match req {
        InboxRequest::Delete(_, _) => {
            warn!("unimplemented Delete");
        },
        InboxRequest::Follow { activity, followed, mut conn, outbound } => {
            let kid = key_id(&followed);
            let http = HttpWrapper::new(http, &kid);
            
            let follower = http.get_person(&activity.actor).await.unwrap();

            let follow = db::Follow {
                id: ObjectUri(
                    format!("https://ferri.amy.mov/activities/{}", crate::new_id())
                ),
                follower: follower.obj.id.clone(),
                followed: followed.actor.id.clone()
            };

            make::new_follow(follow, &mut conn).await.unwrap();

            let activity = ap::AcceptActivity {
                obj: Object {
                    context: as_context(),
                    id: ObjectUri(
                        format!("https://ferri.amy.mov/activities/{}", crate::new_id())
                    )
                },
                ty: ap::ActivityType::Accept,
                object: activity.obj.id.0.clone(),
                actor: followed.actor.id.0.clone()
            };
            
            let msg = QueueMessage::Outbound(
                OutboxRequest::Accept(activity, kid, follower)
            );
            
            outbound.send(msg).await;
        },
        InboxRequest::Create(activity, user, mut conn) => {
            let id = key_id(&user);
            let http = HttpWrapper::new(http, &id);
            let person = http.get_person(&activity.actor).await.unwrap();
            let rmt = person.remote_info();
            
            let post = activity.object;
            let post_id = crate::ap::new_id();

            let created_at = DateTime::parse_from_rfc3339(&activity.ts)
                .map(|dt| dt.to_utc())
                .unwrap();

            let actor_uri = person.obj.id;
            
            let actor = db::Actor {
                id: actor_uri,
                inbox: person.inbox,
                outbox: person.outbox
            };

            make::new_actor(actor.clone(), &mut conn)
                .await
                .unwrap();
            
            let user = get::user_by_actor_uri(actor.id.clone(), &mut conn)
                .await
                .unwrap_or_else(|_| {
                    db::User {
                        id: ObjectUuid(crate::new_id()),
                        actor,
                        username: person.preferred_username,
                        display_name: person.name,
                        acct: rmt.acct,
                        remote: rmt.is_remote,
                        url: rmt.web_url,
                        created_at: crate::ap::now(),
                        icon_url: person.icon.map(|ic| ic.url)
                            .unwrap_or("https//ferri.amy.mov/assets/pfp.png".to_string()),
                        posts: db::UserPosts {
                            last_post_at: None
                        }
                    }
                });

            make::new_user(user.clone(), &mut conn)
                .await
                .unwrap();

            let post = db::Post {
                id: ObjectUuid(post_id),
                uri: post.obj.id,
                user,
                content: post.content,
                created_at,
                boosted_post: None
            };

            
            make::new_post(post, &mut conn)
                .await
                .unwrap();
        },
        InboxRequest::Like(_, _) => {
            warn!("unimplemented Like in inbox");
        },
        InboxRequest::Boost(_, _) => {
            warn!("unimplemented Boost in inbox");
        },
    }
}
