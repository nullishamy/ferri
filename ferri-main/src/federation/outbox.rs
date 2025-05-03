use serde::{Deserialize, Serialize};
use tracing::info;
use std::fmt::Debug;
use crate::{ap::http::HttpClient, federation::http::HttpWrapper, types::{ap, ObjectContext}};

#[derive(Debug)]
pub enum OutboxRequest {
    // FIXME: Make the String (key_id) nicer
    //        Probably store it in the DB and pass a db::User here
    Accept(ap::AcceptActivity, String, ap::Person)
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
                published: crate::ap::new_ts()
            };

            let res = http
                .post_activity(&person.inbox, activity)
                .await
                .unwrap();
            
            info!("accept res {}", res);
        },
    }
}
