use rocket::http::{ContentType, MediaType};

pub mod oauth;
pub mod user;

pub mod api;
pub mod custom;
pub mod inbox;
pub mod well_known;

fn activity_type() -> ContentType {
    ContentType(MediaType::new("application", "activity+json"))
}
