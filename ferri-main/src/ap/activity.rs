use crate::ap::{Actor, User, http};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Sqlite;
use std::fmt::Debug;
use tracing::{Level, event};

#[derive(Debug, Clone)]
pub enum ActivityType {
    Follow,
    Accept,
    Create,
    Unknown,
}

impl ActivityType {
    fn to_raw(self) -> String {
        match self {
            ActivityType::Follow => "Follow".to_string(),
            ActivityType::Accept => "Accept".to_string(),
            ActivityType::Create => "Create".to_string(),
            ActivityType::Unknown => "FIXME".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Activity<T: Serialize + Debug> {
    pub id: String,
    pub ty: ActivityType,
    pub object: T,
    pub published: DateTime<Utc>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
}

impl<T: Serialize + Debug + Default> Default for Activity<T> {
    fn default() -> Self {
        Self {
            id: Default::default(),
            ty: ActivityType::Unknown,
            object: Default::default(),
            published: Utc::now(),
            to: Default::default(),
            cc: Default::default(),
        }
    }
}

pub type KeyId = String;

#[derive(Debug, Clone)]
pub struct OutgoingActivity<T: Serialize + Debug> {
    pub signed_by: KeyId,
    pub req: Activity<T>,
    pub to: Actor,
}

impl<T: Serialize + Debug> OutgoingActivity<T> {
    pub async fn save(&self, conn: impl sqlx::Executor<'_, Database = Sqlite>) {
        let ty = self.req.ty.clone().to_raw();
        let actor_id = self.to.id();

        sqlx::query!(
            r#"
            INSERT INTO activity (id, ty, actor_id)
            VALUES (?1, ?2, ?3) 
        "#,
            self.req.id,
            ty,
            actor_id
        )
        .execute(conn)
        .await
        .unwrap();
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct RawActivity<T: Serialize + Debug> {
    #[serde(rename = "@context")]
    #[serde(skip_deserializing)]
    context: String,

    id: String,
    #[serde(rename = "type")]
    ty: String,

    actor: String,
    object: T,
    published: String,
}

type OutboxTransport = http::HttpClient;
pub struct Outbox<'a> {
    user: User,
    transport: &'a OutboxTransport,
}

impl<'a> Outbox<'a> {
    pub fn user(&self) -> &User {
        &self.user
    }

    pub async fn post<T: Serialize + Debug>(&self, activity: OutgoingActivity<T>) {
        event!(Level::INFO, ?activity, "activity in outbox");

        let raw = RawActivity {
            context: "https://www.w3.org/ns/activitystreams".to_string(),
            id: activity.req.id.clone(),
            ty: activity.req.ty.to_raw(),
            actor: self.user.actor().id().to_string(),
            object: activity.req.object,
            published: activity.req.published.to_rfc3339(),
        };

        let outbox_res = self
            .transport
            .post(activity.to.inbox())
            .activity()
            .json(&raw)
            .sign(&activity.signed_by)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        event!(
            Level::DEBUG,
            outbox_res,
            activity = activity.req.id,
            "got response for outbox dispatch"
        );
    }

    pub fn for_user(user: User, transport: &'a OutboxTransport) -> Outbox<'a> {
        Outbox { user, transport }
    }
}
