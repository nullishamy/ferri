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
