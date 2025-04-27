use serde::{Serialize, Deserialize};

mod convert;
pub use convert::*;

pub const AS_CONTEXT_RAW: &'static str = "https://www.w3.org/ns/activitystreams";
pub fn as_context() -> ObjectContext {
    ObjectContext::Str(AS_CONTEXT_RAW.to_string())
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum ObjectContext {
    Str(String),
    Vec(Vec<serde_json::Value>),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ObjectUri(String);

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ObjectUuid(String);

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Object {
    #[serde(rename = "@context")]
    context: ObjectContext,
    id: ObjectUri,
}

pub mod db {
    use serde::{Serialize, Deserialize};
    use super::*;
    
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct Actor {
        pub id: ObjectUri,
        pub inbox: String,
        pub outbox: String,
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct User {
        pub id: ObjectUuid,
        pub actor_id: ObjectUri,
        pub username: String,
        pub display_name: String
    }
}

pub mod ap {
    use serde::{Serialize, Deserialize};
    use super::*;
    
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct Actor {
        #[serde(flatten)]
        pub obj: Object,
        
        pub inbox: String,
        pub outbox: String,
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct Person {
        #[serde(flatten)]
        pub obj: Object,

        pub following: String,
        pub followers: String,
        
        pub summary: String,
        pub inbox: String,
        pub outbox: String,
        
        pub preferred_username: String,
        pub name: String,
        
        pub public_key: Option<UserKey>,
    }
    
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct UserKey {
        pub id: String,
        pub owner: String,
        
        #[serde(rename = "publicKeyPem")]
        pub public_key: String,
    }
}

pub mod api {
    use serde::{Serialize, Deserialize};
    use super::*;
    
    // API will not really use actors so treat them as DB actors
    // until we require specificity
    pub type Actor = db::Actor;

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct Account {
        pub id: ObjectUuid,
        pub username: String,
        pub acct: String,
        pub display_name: String,
        
        pub locked: bool,
        pub bot: bool,
        
        pub created_at: String,
        pub attribution_domains: Vec<String>,
        
        pub note: String,
        pub url: String,
        
        pub avatar: String,
        pub avatar_static: String,
        pub header: String,
        pub header_static: String,
        
        pub followers_count: i64,
        pub following_count: i64,
        pub statuses_count: i64,
        pub last_status_at: String,
        
        pub emojis: Vec<Emoji>,
        pub fields: Vec<CustomField>,
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct Emoji {
        pub shortcode: String,
        pub url: String,
        pub static_url: String,
        pub visible_in_picker: bool,
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct CustomField {
        pub name: String,
        pub value: String,
        pub verified_at: Option<String>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn ap_actor_to_db() {
        let domain = "https://example.com";
        
        let ap = ap::Actor {
            obj: Object {
                context: as_context(),
                id: ObjectUri(format!("{}/users/sample", domain)),
            },
            inbox: format!("{}/users/sample/inbox", domain),
            outbox: format!("{}/users/sample/outbox", domain),
        };

        let db: db::Actor = ap.into();
        
        assert_eq!(db, db::Actor {
            id: ObjectUri("https://example.com/users/sample".to_string()),
            inbox: "https://example.com/users/sample/inbox".to_string(),
            outbox: "https://example.com/users/sample/outbox".to_string(),
        });
    }
}
