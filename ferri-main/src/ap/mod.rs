use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::Sqlite;

use std::fmt::Debug;
pub mod http;

#[derive(Debug, Clone)]
pub struct Actor {
    id: String,
    inbox: String,
    outbox: String,
}

impl Actor {
    pub fn from_raw(id: String, inbox: String, outbox: String) -> Self {
        Self { id, inbox, outbox }
    }
}

#[derive(Debug, Clone)]
pub struct User {
    id: String,
    username: String,
    actor: Actor,
    display_name: String,
}

impl User {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn actor_id(&self) -> &str {
        &self.actor.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn actor(&self) -> &Actor {
        &self.actor
    }

    pub async fn from_id(
        uuid: &str,
        conn: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> User {
        let user = sqlx::query!(
            r#"
                SELECT u.*, a.id as "actor_own_id", a.inbox, a.outbox
                FROM user u
                INNER JOIN actor a ON u.actor_id = a.id
                WHERE u.id = ?1
            "#,
            uuid
        )
        .fetch_one(conn)
        .await
        .unwrap();
        User {
            id: user.id,
            username: user.username,
            actor: Actor {
                id: user.actor_own_id,
                inbox: user.inbox,
                outbox: user.outbox,
            },
            display_name: user.display_name,
        }
    }

    pub async fn from_username(
        username: &str,
        conn: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> User {
        let user = sqlx::query!(
            r#"
                SELECT u.*, a.id as "actor_own_id", a.inbox, a.outbox
                FROM user u
                INNER JOIN actor a ON u.actor_id = a.id
                WHERE username = ?1
            "#,
            username
        )
        .fetch_one(conn)
        .await
        .unwrap();
        User {
            id: user.id,
            username: user.username,
            actor: Actor {
                id: user.actor_own_id,
                inbox: user.inbox,
                outbox: user.outbox,
            },
            display_name: user.display_name,
        }
    }

    pub async fn from_actor_id(
        actor_id: &str,
        conn: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> User {
        let user = sqlx::query!(
            r#"
                SELECT u.*, a.id as "actor_own_id", a.inbox, a.outbox
                FROM user u
                INNER JOIN actor a ON u.actor_id = a.id
                WHERE actor_id = ?1
            "#,
            actor_id
        )
        .fetch_one(conn)
        .await
        .unwrap();
        User {
            id: user.id,
            username: user.username,
            actor: Actor {
                id: user.actor_own_id,
                inbox: user.inbox,
                outbox: user.outbox,
            },
            display_name: user.display_name,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ActivityType {
    Follow,
    Accept,
    Create,
    Unknown
}

impl ActivityType {
    fn to_raw(self) -> String {
        match self {
            ActivityType::Follow => "Follow".to_string(),
            ActivityType::Accept => "Accept".to_string(),
            ActivityType::Create => "Create".to_string(),
            ActivityType::Unknown => "FIXME".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Activity<T : Serialize + Debug> {
    pub id: String,
    pub ty: ActivityType,
    pub object: T,
    pub published: DateTime<Local>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
}

impl <T : Serialize + Debug + Default> Default for Activity<T> {
    fn default() -> Self {
        Self {
            id: Default::default(),
            ty: ActivityType::Unknown,
            object: Default::default(),
            published: Local::now(),
            to: Default::default(),
            cc: Default::default(),
        }
    }
}

pub type KeyId = String;

#[derive(Debug, Clone)]
pub struct OutgoingActivity<T : Serialize + Debug> {
    pub signed_by: KeyId,
    pub req: Activity<T>,
    pub to: Actor,
}

impl <T : Serialize + Debug> OutgoingActivity<T> {
    pub async fn save(&self, conn: impl sqlx::Executor<'_, Database = Sqlite>) {
        let ty = self.req.ty.clone().to_raw();
        sqlx::query!(
            r#"
            INSERT INTO activity (id, ty, actor_id)
            VALUES (?1, ?2, ?3) 
        "#,
            self.req.id,
            ty,
            self.to.id
        )
        .execute(conn)
        .await
        .unwrap();
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct RawActivity<T : Serialize + Debug> {
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

    pub async fn post<T : Serialize + Debug>(&self, activity: OutgoingActivity<T>) {
        dbg!(&activity);
        let raw = RawActivity {
            context: "https://www.w3.org/ns/activitystreams".to_string(),
            id: activity.req.id,
            ty: activity.req.ty.to_raw(),
            actor: self.user.actor.id.clone(),
            object: activity.req.object,
            published: activity.req.published.to_rfc3339(),
        };

        dbg!(&raw);

        let follow_res = self
            .transport
            .post(activity.to.inbox)
            .activity()
            .json(&raw)
            .sign(&activity.signed_by)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        dbg!(follow_res);
    }

    pub fn for_user(user: User, transport: &'a OutboxTransport) -> Outbox<'a> {
        Outbox { user, transport }
    }
}
