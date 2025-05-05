use crate::types::ap;
use crate::types::api;
use crate::types::db;

use crate::types::{Object, ObjectUri, as_context};

use super::ap::ActivityType;

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

            avatar: val.icon_url.clone(),
            avatar_static: val.icon_url.clone(),
            header: val.icon_url.clone(),
            header_static: val.icon_url,

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
            ty: ActivityType::Person,
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
            icon: None
        }
    }
}

impl From<db::Post> for api::Status {
    fn from(value: db::Post) -> api::Status {
        api::Status {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            in_reply_to_id: None,
            in_reply_to_account_id: None,
            sensitive: false,
            spoiler_text: String::new(),
            visibility: "Public".to_string(),
            language: "en-GB".to_string(),
            uri: value.uri.clone(),
            url: value.uri.0.to_string(),
            replies_count: 0,
            reblogs_count: 0,
            favourites_count: 0,
            favourited: false,
            reblogged: false,
            muted: false,
            bookmarked: false,
            content: value.content,
            reblog: value.boosted_post.map(|p| {
                // Probably a better way to do this without reboxing but whatever...
                let p: db::Post = *p;
                let p: api::Status = p.into();
                Box::new(p)
            }),
            application: None,
            account: value.user.into(),
            media_attachments: value.attachments
                .into_iter()
                .map(|at| api::StatusAttachment {
                    id: at.id,
                    ty: "image".to_string(),
                    url: at.url,
                    description: at.alt.unwrap_or(String::new())
                })
                .collect(),
            mentions: vec![],
            tags: vec![],
            emojis: vec![],
            card: None,
            poll: None
        }
    }
}

