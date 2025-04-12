use sqlx::Sqlite;
use std::fmt::Debug;

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

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn inbox(&self) -> &str {
        &self.inbox
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

    pub fn uri(&self) -> String {
        format!("https://ferri.amy.mov/users/{}", self.id())
    }

    pub async fn from_id(uuid: &str, conn: impl sqlx::Executor<'_, Database = Sqlite>) -> User {
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
