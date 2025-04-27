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
}

pub mod api {
    use serde::{Serialize, Deserialize};
    use super::*;
    
    // API will not really use actors so treat them as DB actors
    // until we require specificity
    pub type Actor = db::Actor;
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
