use rocket::http::{ContentType, MediaType};

pub mod user;
pub mod oauth;

pub mod api;
pub mod well_known;
pub mod custom;
pub mod inbox;

fn activity_type() -> ContentType {
    ContentType(MediaType::new("application", "activity+json"))
}