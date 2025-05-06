use crate::types::{ap, as_context, db, get, make, Object, ObjectUri, ObjectUuid};

use super::http::{HttpClient, HttpWrapper};
use super::outbox::OutboxRequest;
use super::QueueMessage;

use chrono::DateTime;
use tracing::{warn, error, Level, event};

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
    Boost(ap::BoostActivity, db::User, sqlx::SqliteConnection)
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

            let actor = db::Actor {
                id: follower.obj.id.clone(),
                inbox: follower.inbox.clone(),
                outbox: follower.outbox.clone()
            };

            make::new_actor(actor, &mut conn).await.unwrap();

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
            let post_id = crate::new_id();

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
                    let id = crate::new_id();
                    db::User {
                        id: ObjectUuid(id.clone()),
                        actor,
                        username: person.preferred_username,
                        display_name: person.name,
                        acct: rmt.acct,
                        remote: rmt.is_remote,
                        url: rmt.web_url,
                        created_at: crate::now(),
                        icon_url: person.icon.map(|ic| ic.url)
                            .unwrap_or("https//ferri.amy.mov/assets/pfp.png".to_string()),
                        posts: db::UserPosts {
                            last_post_at: None
                        },
                        key_id: format!(
                            "https://ferri.amy.mov/users/{}#main-key",
                            id
                        )
                    }
                });

            make::new_user(user.clone(), &mut conn)
                .await
                .unwrap();

            let attachments = post.attachment
                .into_iter()
                .map(|at| {
                    db::Attachment {
                        id: ObjectUuid(crate::new_id()),
                        post_id: ObjectUuid(post_id.clone()),
                        url: at.url,
                        media_type: Some(at.media_type),
                        sensitive: at.sensitive,
                        alt: at.summary
                    }
                })
                .collect::<Vec<_>>();

            let post = db::Post {
                id: ObjectUuid(post_id),
                uri: post.obj.id,
                user,
                content: post.content,
                created_at,
                attachments,
                boosted_post: None
            };
            
            make::new_post(post, &mut conn)
                .await
                .unwrap();
        },
        InboxRequest::Like(_, _) => {
            warn!("unimplemented Like in inbox");
        },
        InboxRequest::Boost(activity, target, mut conn) => {
            let id = key_id(&target);
            let http = HttpWrapper::new(http, &id);
            let person = http.get_person(&activity.actor).await.unwrap();
            let rmt = person.remote_info();

            let boosted_note = http.get_note(&activity.object).await.unwrap();
            let boosted_author = if let Some(attributed_to) = &boosted_note.attributed_to {
                http.get_person(attributed_to).await.map_err(|e| {
                    error!("failed to fetch attributed_to {}: {}",
                           attributed_to,
                           e.to_string()
                    );
                    ()
                })
                .ok()
            } else {
                None
            }.unwrap();

            let boosted_rmt = boosted_author.remote_info();

            event!(Level::INFO,
                   boosted_by = rmt.acct,
                   op = boosted_rmt.acct,
                   "recording boost"
            );

            let boosted_post = {
                let actor_uri = boosted_author.obj.id;
                let actor = db::Actor {
                    id: actor_uri,
                    inbox: boosted_author.inbox,
                    outbox: boosted_author.outbox
                };

                make::new_actor(actor.clone(), &mut conn)
                    .await
                    .unwrap();
                
                let user = get::user_by_actor_uri(actor.id.clone(), &mut conn)
                    .await
                    .unwrap_or_else(|_| {
                        let id = crate::new_id();
                        db::User {
                            id: ObjectUuid(id.clone()),
                            actor,
                            username: boosted_author.preferred_username,
                            display_name: boosted_author.name,
                            acct: boosted_rmt.acct,
                            remote: boosted_rmt.is_remote,
                            url: boosted_rmt.web_url,
                            // FIXME: Come from boosted_author
                            created_at: crate::now(),
                            icon_url: boosted_author.icon.map(|ic| ic.url)
                                .unwrap_or("https//ferri.amy.mov/assets/pfp.png".to_string()),
                            posts: db::UserPosts {
                                last_post_at: None
                            },
                            key_id: format!(
                                "https://ferri.amy.mov/users/{}#main-key",
                                id
                            )
                        }
                    });

                
                make::new_user(user.clone(), &mut conn)
                    .await
                    .unwrap();
                
                let id = crate::new_id();
                let created_at = DateTime::parse_from_rfc3339(&boosted_note.ts)
                    .map(|dt| dt.to_utc())
                    .unwrap();

                let attachments = boosted_note.attachment
                    .into_iter()
                    .map(|at| {
                        db::Attachment {
                            id: ObjectUuid(crate::new_id()),
                            post_id: ObjectUuid(id.clone()),
                            url: at.url,
                            media_type: Some(at.media_type),
                            sensitive: at.sensitive,
                            alt: at.summary
                        }
                    })
                    .collect::<Vec<_>>();
                
                db::Post {
                    id: ObjectUuid(id),
                    uri: boosted_note.obj.id,
                    user,
                    attachments,
                    content: boosted_note.content,
                    created_at,
                    boosted_post: None

                }
            };

            make::new_post(boosted_post.clone(), &mut conn).await.unwrap();

            let base_note = {
                let actor_uri = person.obj.id.clone();
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
                        let id = crate::new_id();
                        db::User {
                            id: ObjectUuid(id.clone()),
                            actor,
                            username: person.preferred_username,
                            display_name: person.name,
                            acct: rmt.acct,
                            remote: rmt.is_remote,
                            url: rmt.web_url,
                            created_at: crate::now(),
                            icon_url: person.icon.map(|ic| ic.url)
                                .unwrap_or("https//ferri.amy.mov/assets/pfp.png".to_string()),
                            posts: db::UserPosts {
                                last_post_at: None
                            },
                            key_id: format!(
                                "https://ferri.amy.mov/users/{}#main-key",
                                id
                            )
                        }
                    });

                
                make::new_user(user.clone(), &mut conn)
                    .await
                    .unwrap();
                
                let id = crate::new_id();
                let created_at = DateTime::parse_from_rfc3339(&activity.published)
                    .map(|dt| dt.to_utc())
                    .unwrap();
                
                db::Post {
                    id: ObjectUuid(id.clone()),
                    uri: ObjectUri(
                        format!("https://ferri.amy.mov/users/{}/posts/{}",
                                person.obj.id.0,
                                id
                        )
                    ),
                    user,
                    attachments: vec![],
                    content: String::new(),
                    created_at,
                    boosted_post: Some(Box::new(boosted_post.clone()))
                }
            };

            make::new_post(base_note, &mut conn).await.unwrap();
        },
    }
}
