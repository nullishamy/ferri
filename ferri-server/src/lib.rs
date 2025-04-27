use endpoints::{
    api::{self, timeline},
    custom, inbox, oauth, user, well_known,
};

use tracing_subscriber::fmt;

use main::ap;

use main::ap::http;
use main::config::Config;
use rocket::{
    Build, Request, Rocket, build, get,
    http::{ContentType, Status},
    request::{FromRequest, Outcome},
    routes,
};
use rocket_db_pools::{Connection, Database, sqlx};

mod cors;
mod endpoints;
mod types;
mod http_wrapper;

#[derive(Database)]
#[database("sqlite_ferri")]
pub struct Db(sqlx::SqlitePool);

#[get("/")]
async fn user_profile() -> (ContentType, &'static str) {
    (ContentType::HTML, "<p>hello</p>")
}

#[get("/activities/<_activity>")]
async fn activity_endpoint(_activity: String) {

}

#[derive(Debug)]
pub struct AuthenticatedUser {
    pub username: String,
    pub id: String,
    pub token: String,
    pub actor_id: String,
}

#[derive(Debug)]
pub enum LoginError {
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for AuthenticatedUser {
    type Error = LoginError;
    async fn from_request(request: &'a Request<'_>) -> Outcome<AuthenticatedUser, LoginError> {
        let token = request.headers().get_one("Authorization");

        if let Some(token) = token {
            let token = token
                .strip_prefix("Bearer")
                .map(|s| s.trim())
                .unwrap_or(token);

            let mut conn = request.guard::<Connection<Db>>().await.unwrap();
            let auth = sqlx::query!(
                r#"
                SELECT *
                FROM auth a
                INNER JOIN user u ON a.user_id = u.id
                WHERE token = ?1
            "#,
                token
            )
            .fetch_one(&mut **conn)
            .await;

            if let Ok(auth) = auth {
                return Outcome::Success(AuthenticatedUser {
                    token: auth.token,
                    id: auth.id,
                    username: auth.display_name,
                    actor_id: auth.actor_id,
                });
            }
        }

        Outcome::Forward(Status::Unauthorized)
    }
}

pub struct OutboundQueue(pub ap::QueueHandle);
pub struct InboundQueue(pub ap::QueueHandle);

pub fn launch(cfg: Config) -> Rocket<Build> {
    let format = fmt::format()
        .with_ansi(true)
        .without_time()
        .with_level(true)
        .with_target(false)
        .with_thread_names(false)
        .with_source_location(false)
        .compact();
    
    tracing_subscriber::fmt()
        .event_format(format)
        .with_writer(std::io::stdout)
        .init();

    let outbound = ap::RequestQueue::new("outbound");
    let outbound_handle = outbound.spawn();

    let inbound = ap::RequestQueue::new("inbound");
    let inbound_handle = inbound.spawn();

    let http_client = http::HttpClient::new();
    build()
        .manage(cfg)
        .manage(http_client)
        .manage(OutboundQueue(outbound_handle))
        .manage(InboundQueue(inbound_handle))
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
                api::status::status_context,
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
