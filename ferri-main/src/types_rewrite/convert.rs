use crate::types_rewrite::api;
use crate::types_rewrite::ap;
use crate::types_rewrite::db;

use crate::types_rewrite::{Object, as_context};

impl From<db::Actor> for ap::Actor {
    fn from(val: db::Actor) -> ap::Actor {
        ap::Actor {
            obj: Object {
                context: as_context(),
                id: val.id
            },
            inbox: val.inbox,
            outbox: val.outbox
        }
    }
}

impl From<ap::Actor> for db::Actor {
    fn from(val: ap::Actor) -> db::Actor {
        db::Actor {
            id: val.obj.id,
            inbox: val.inbox,
            outbox: val.outbox
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
