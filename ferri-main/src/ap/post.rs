use crate::ap;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::Sqlite;

const POST_TYPE: &'static str = "Post";

#[derive(Clone)]
pub struct Post {
    id: String,
    from: ap::User,
    ts: DateTime<Utc>,
    content: String,

    to: Vec<String>,
    cc: Vec<String>,
}

impl Post {
    pub fn from_parts(id: String, content: String, from: ap::User) -> Self {
        Self {
            id,
            content,
            from,
            ts: ap::now(),
            to: vec![],
            cc: vec![],
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn created_at(&self) -> String {
        self.ts.to_rfc3339()
    }

    pub fn uri(&self) -> String {
        format!(
            "https://ferri.amy.mov/users/{}/posts/{}",
            self.from.id(),
            self.id
        )
    }

    pub async fn save(&self, conn: impl sqlx::Executor<'_, Database = Sqlite>) {
        let ts = self.ts.to_rfc3339();
        let user_id = self.from.id();
        let post_id = self.id();
        let uri = self.uri();
        let content = self.content.clone();

        sqlx::query!(
            r#"
                INSERT INTO post (id, uri, user_id, content, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            post_id,
            uri,
            user_id,
            content,
            ts
        )
        .execute(conn)
        .await
        .unwrap();
    }

    pub fn to(mut self, recipient: String) -> Self {
        self.to.push(recipient);
        self
    }

    pub fn cc(mut self, recipient: String) -> Self {
        self.cc.push(recipient);
        self
    }

    pub fn to_ap(self) -> APPost {
        APPost {
            context: ap::AS_CONTEXT.to_string(),
            id: self.uri(),
            ty: POST_TYPE.to_string(),
            ts: self.ts.to_rfc3339(),
            content: self.content,
            to: self.to,
            cc: self.cc,
        }
    }
}

#[derive(Serialize, Debug, Default)]
pub struct APPost {
    #[serde(rename = "@context")]
    #[serde(skip_deserializing)]
    context: String,
    id: String,

    #[serde(rename = "type")]
    ty: String,

    #[serde(rename = "published")]
    ts: String,

    content: String,
    to: Vec<String>,
    cc: Vec<String>,
}
