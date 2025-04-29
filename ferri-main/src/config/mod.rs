use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub host: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub server: ServerConfig,
}

impl Config {
    pub fn host(&self) -> &str {
        &self.server.host
    }

    pub fn user_url(&self, user_uuid: &str) -> String {
        format!("{}/users/{}", self.host(), user_uuid)
    }

    pub fn user_web_url(&self, user_name: &str) -> String {
        format!("{}/{}", self.host(), user_name)
    }

    pub fn followers_url(&self, user_uuid: &str) -> String {
        format!("{}/followers", self.user_url(user_uuid))
    }

    pub fn following_url(&self, user_uuid: &str) -> String {
        format!("{}/following", self.user_url(user_uuid))
    }

    pub fn inbox_url(&self, user_uuid: &str) -> String {
        format!("{}/inbox", self.user_url(user_uuid))
    }

    pub fn outbox_url(&self, user_uuid: &str) -> String {
        format!("{}/outbox", self.user_url(user_uuid))
    }

    pub fn post_url(&self, poster_uuid: &str, post_uuid: &str) -> String {
        format!("{}/{}", self.user_url(poster_uuid), post_uuid)
    }

    pub fn activity_url(&self, activity_uuid: &str) -> String {
        format!("{}/activities/{}", self.host(), activity_uuid)
    }
}
