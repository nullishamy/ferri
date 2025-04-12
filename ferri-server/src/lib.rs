use main::ap::http;
use rocket::{
    build, get, http::ContentType, request::{FromRequest, Outcome}, routes, Build, Request, Rocket
};
use endpoints::{api::{self, timeline}, oauth, well_known, custom, user, inbox};
use rocket_db_pools::{sqlx, Database};

mod cors;
mod types;
mod endpoints;

#[derive(Database)]
#[database("sqlite_ferri")]
pub struct Db(sqlx::SqlitePool);

#[get("/")]
async fn user_profile() -> (ContentType, &'static str) {
    (ContentType::HTML, "<p>hello</p>")
}

#[get("/activities/<activity>")]
async fn activity_endpoint(activity: String) {
    dbg!(activity);
}

#[derive(Debug)]
struct AuthenticatedUser {
    username: String,
    actor_id: String
}

#[derive(Debug)]
enum LoginError {
    InvalidData,
    UsernameDoesNotExist,
    WrongPassword,
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for AuthenticatedUser {
    type Error = LoginError;
    async fn from_request(request: &'a Request<'_>) -> Outcome<AuthenticatedUser, LoginError> {
        let token = request.headers().get_one("Authorization").unwrap();
        let token = token.strip_prefix("Bearer").map(|s| s.trim()).unwrap_or(token);
        Outcome::Success(AuthenticatedUser {
            username: token.to_string(),
            actor_id: format!("https://ferri.amy.mov/users/{}", token)
        })
    }
}

pub fn launch() -> Rocket<Build> {
    let http_client = http::HttpClient::new();
    build()
        .manage(http_client)
        .attach(Db::init())
        .attach(cors::CORS)
        .mount("/assets", rocket::fs::FileServer::from("./assets"))
        .mount(
            "/",
            routes![
                custom::test,
                user::inbox,
                user::outbox,
                user::user,
                user::followers,
                user::following,
                user::post,
                oauth::authorize,
                oauth::new_token,
                cors::options_req,
                activity_endpoint,
                well_known::webfinger,
                well_known::host_meta,
                inbox::inbox,
                user_profile,
            ],
        )
        .mount("/api/v2", routes![api::instance::instance])
        .mount(
            "/api/v1",
            routes![
                api::status::new_status,
                api::status::new_status_json,
                api::user::new_follow,
                api::user::statuses,
                api::user::account,
                api::apps::new_app,
                api::preferences::preferences,
                api::user::verify_credentials,
                custom::finger_account,
                timeline::home
            ],
        )
}
