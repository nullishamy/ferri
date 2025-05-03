use main::{
    federation::{
        QueueMessage,
        inbox::InboxRequest
    },
    types::{ap, get, ObjectUuid}
};
use rocket::{State, post, serde::json::serde_json};
use rocket_db_pools::Connection;
use serde::de::DeserializeOwned;
use tracing::{debug, event, span, warn, Instrument, Level};

use crate::{Db, InboundQueue, OutboundQueue};

fn deser<T : DeserializeOwned>(body: &str) -> T {
    serde_json::from_str(body).unwrap()
}

#[post("/users/<user_uuid>/inbox", data = "<body>")]
pub async fn inbox(
    mut db: Connection<Db>,
    queue: &State<InboundQueue>,
    outbound: &State<OutboundQueue>,
    user_uuid: &str,
    body: String
) {
    let user = get::user_by_id(
        ObjectUuid(user_uuid.to_string()),
        &mut db
    )
        .await
        .unwrap();
    
    debug!("body in inbox: {}", body);

    let min = deser::<ap::MinimalActivity>(&body);
    let span = span!(Level::INFO, "user-inbox", user_id = user_uuid);

    let conn = db.into_inner();
    let conn = conn.detach();

    async move {
        event!(Level::INFO, ?min, "received an activity");

        match min.ty {
            ap::ActivityType::Delete => {
                let activity = deser::<ap::DeleteActivity>(&body);
                let msg = QueueMessage::Inbound(
                    InboxRequest::Delete(activity, user)
                );
                
                queue.0.send(msg).await;
            }
            ap::ActivityType::Follow => {
                let activity = deser::<ap::FollowActivity>(&body);
                let msg = QueueMessage::Inbound(
                    InboxRequest::Follow {
                        activity,
                        followed: user,
                        conn,
                        outbound: outbound.0.clone()
                    }
                );
                
                queue.0.send(msg).await;
            }
            ap::ActivityType::Create => {
                let activity = deser::<ap::CreateActivity>(&body);
                let msg = QueueMessage::Inbound(
                    InboxRequest::Create(activity, user, conn)
                );
                
                queue.0.send(msg).await;
            }
            ap::ActivityType::Like => {
                let activity = deser::<ap::LikeActivity>(&body);
                let msg = QueueMessage::Inbound(
                    InboxRequest::Like(activity, user)
                );
                
                queue.0.send(msg).await;
            }
            ap::ActivityType::Announce => {
                let activity = deser::<ap::BoostActivity>(&body);
                let msg = QueueMessage::Inbound(
                    InboxRequest::Boost(activity, user)
                );
                
                queue.0.send(msg).await;
            },
            unimpl => {
                warn!("unimplemented {:?}", unimpl);
            }
        }
    }
    .instrument(span)
    .await;
}
