use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use tracing::info;
use std::fmt::Debug;
use crate::{federation::http::HttpWrapper, types::{ap::{self, ActivityType}, as_context, db, make, Object, ObjectContext, ObjectUri}};

use super::http::HttpClient;

#[derive(Debug)]
pub enum OutboxRequest {
    // FIXME: Make the String (key_id) nicer
    //        Probably store it in the DB and pass a db::User here
    Accept(ap::AcceptActivity, String, ap::Person),
    Status(db::Post, String),
    Follow {
        follower: db::User,
        followed: db::User,
        conn: SqliteConnection
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PreparedActivity<T: Serialize + Debug> {
    #[serde(rename = "@context")]
    context: ObjectContext,

    id: String,
    #[serde(rename = "type")]
    ty: ap::ActivityType,

    actor: String,
    object: T,
    published: String,
}

pub async fn handle_outbox_request(
    req: OutboxRequest,
    http: &HttpClient
) {
    match req {
        OutboxRequest::Accept(activity, key_id, person) => {
            let http = HttpWrapper::new(http, &key_id);
            
            info!("accepting {}", activity.object);
            let activity = PreparedActivity {
                context: activity.obj.context,
                id: activity.obj.id.0,
                ty: activity.ty,
                actor: activity.actor,
                object: activity.object,
                published: crate::now_str(),
            };

            let res = http
                .post_activity(&person.inbox, activity)
                .await
                .unwrap();
            
            info!("accept res {}", res);
        },
        OutboxRequest::Status(post, key_id) => {
            // FIXME: Take a list of who we should send to
            //        for now we only propogate to my main instance

            let http = HttpWrapper::new(http, &key_id);
            
            let activity = PreparedActivity {
                context: as_context(),
                id: format!("https://ferri.amy.mov/activities/{}", crate::new_id()),
                ty: ActivityType::Create,
                actor: post.user.actor.id.0.clone(),
                object: ap::Post {
                    obj: Object {
                        id: ObjectUri(
                            format!(
                                "https://ferri.amy.mov/users/{}/posts/{}",
                                post.user.id.0,
                                post.id.0
                            )
                        ),
                        context: as_context()
                    },
                    ty: ActivityType::Note,
                    ts: post.created_at.to_rfc3339(),
                    content: post.content,
                    to: vec![format!("https://ferri.amy.mov/users/{}/followers", post.user.id.0)],
                    cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
                    attachment: vec![],
                    attributed_to: Some(post.user.actor.id.0)
                },
                published: crate::now_str(),
            };

            let res = http
                .post_activity("https://fedi.amy.mov/users/9zkygethkdw60001/inbox", activity)
                .await
                .unwrap();
            
            info!("status res {}", res);
        }
        OutboxRequest::Follow { follower, followed, mut conn } => {
            let follow = db::Follow {
                id: ObjectUri(format!(
                    "https://ferri.amy.mov/activities/{}",
                    crate::new_id())
                ),
                follower: follower.actor.id.clone(),
                followed: followed.actor.id.clone(),
            };

            make::new_follow(follow, &mut conn)
                .await
                .unwrap();

            let http = HttpWrapper::new(http, &follower.key_id);
            
            let activity = PreparedActivity {
                context: as_context(),
                id: format!(
                    "https://ferri.amy.mov/activities/{}",
                    crate::new_id()
                ),
                ty: ActivityType::Follow,
                actor: follower.actor.id.0,
                object: followed.actor.id.0,
                published: crate::now_str(),
            };

            let res = http
                .post_activity(&followed.actor.inbox, activity)
                .await
                .unwrap();
            
            info!("follow res {}", res);
        },
    }
}
