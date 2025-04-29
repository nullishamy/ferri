use crate::types::ap;
use crate::types::api;
use crate::types::db;

use crate::types::{Object, ObjectUri, as_context};

impl From<db::Actor> for ap::Actor {
    fn from(val: db::Actor) -> ap::Actor {
        ap::Actor {
            obj: Object {
                context: as_context(),
                id: val.id,
            },
            inbox: val.inbox,
            outbox: val.outbox,
        }
    }
}

impl From<ap::Actor> for db::Actor {
    fn from(val: ap::Actor) -> db::Actor {
        db::Actor {
            id: val.obj.id,
            inbox: val.inbox,
            outbox: val.outbox,
        }
    }
}

impl From<db::User> for api::Account {
    fn from(val: db::User) -> api::Account {
        api::Account {
            id: val.id,
            username: val.username,
            acct: val.acct,
            display_name: val.display_name,

            locked: false,
            bot: false,

            created_at: val.created_at.to_rfc3339(),
            attribution_domains: vec![],

            note: "".to_string(),
            url: val.url,

            avatar: "https://ferri.amy.mov/assets/pfp.png".to_string(),
            avatar_static: "https://ferri.amy.mov/assets/pfp.png".to_string(),
            header: "https://ferri.amy.mov/assets/pfp.png".to_string(),
            header_static: "https://ferri.amy.mov/assets/pfp.png".to_string(),

            followers_count: 0,
            following_count: 0,
            statuses_count: 0,
            last_status_at: val.posts.last_post_at.map(|ts| ts.to_rfc3339()),

            emojis: vec![],
            fields: vec![],
        }
    }
}

impl From<db::User> for ap::Person {
    fn from(val: db::User) -> ap::Person {
        ap::Person {
            obj: Object {
                context: as_context(),
                id: ObjectUri(format!("https://ferri.amy.mov/users/{}", val.id.0)),
            },
            following: format!("https://ferri.amy.mov/users/{}/following", val.id.0),
            followers: format!("https://ferri.amy.mov/users/{}/followers", val.id.0),
            summary: format!("ferri {}", val.username),
            inbox: format!("https://ferri.amy.mov/users/{}/inbox", val.id.0),
            outbox: format!("https://ferri.amy.mov/users/{}/outbox", val.id.0),
            preferred_username: val.display_name,
            name: val.username,
            public_key: Some(ap::UserKey {
                id: format!("https://ferri.amy.mov/users/{}#main-key", val.id.0),
                owner: format!("https://ferri.amy.mov/users/{}", val.id.0),
                public_key: include_str!("../../../public.pem").to_string(),
            }),
        }
    }
}
