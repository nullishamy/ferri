use main::ap::{http};
use rocket::{
    build, get, http::ContentType, request::{FromRequest, Outcome}, routes, serde::{
        json::Json, Deserialize, Serialize
    }, Build, Request, Rocket
};


mod cors;
mod types;
mod endpoints;

use endpoints::{api::{self, user::CredentialAcount}, oauth, well_known, custom, user};

use rocket_db_pools::sqlx;
use rocket_db_pools::Database;

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
        Outcome::Success(AuthenticatedUser {
            username: token.to_string(),
            actor_id: format!("https://ferri.amy.mov/users/{}", token)
        })
    }
}

pub type TimelineAccount = CredentialAcount;

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct TimelineStatus {
    id: String,
    created_at: String,
    in_reply_to_id: Option<String>,
    in_reply_to_account_id: Option<String>,
    content: String,
    visibility: String,
    spoiler_text: String,
    sensitive: bool,
    uri: String,
    url: String,
    replies_count: i64,
    reblogs_count: i64,
    favourites_count: i64,
    favourited: bool,
    reblogged: bool,
    muted: bool,
    bookmarked: bool,
    media_attachments: Vec<()>,
    account: TimelineAccount,
}

#[get("/timelines/home?<limit>")]
async fn home_timeline(limit: i64) -> Json<Vec<TimelineStatus>> {
    Json(vec![TimelineStatus {
        id: "1".to_string(),
        created_at: "2025-04-10T22:12:09Z".to_string(),
        in_reply_to_id: None,
        in_reply_to_account_id: None,
        content: "My first post".to_string(),
        visibility: "public".to_string(),
        spoiler_text: "".to_string(),
        sensitive: false,
        uri: "https://ferri.amy.mov/users/amy/posts/1".to_string(),
        url: "https://ferri.amy.mov/users/amy/posts/1".to_string(),
        replies_count: 0,
        reblogs_count: 0,
        favourites_count: 0,
        favourited: false,
        reblogged: false,
        muted: false,
        bookmarked: false,
        media_attachments: vec![],
        account: CredentialAcount {
            id: "https://ferri.amy.mov/users/amy".to_string(),
            username: "amy".to_string(),
            acct: "amy@ferri.amy.mov".to_string(),
            display_name: "amy".to_string(),
            locked: false,
            bot: false,
            created_at: "2025-04-10T22:12:09Z".to_string(),
            attribution_domains: vec![],
            note: "".to_string(),
            url: "https://ferri.amy.mov/@amy".to_string(),
            avatar: "https://i.sstatic.net/l60Hf.png".to_string(),
            avatar_static: "https://i.sstatic.net/l60Hf.png".to_string(),
            header: "https://i.sstatic.net/l60Hf.png".to_string(),
            header_static: "https://i.sstatic.net/l60Hf.png".to_string(),
            followers_count: 1,
            following_count: 1,
            statuses_count: 1,
            last_status_at: "2025-04-10T22:14:34Z".to_string(),
        },
    }])
}

pub fn launch() -> Rocket<Build> {
    let http_client = http::HttpClient::new();
    build()
        .manage(http_client)
        .attach(Db::init())
        .attach(cors::CORS)
        .mount(
            "/",
            routes![
                custom::test,
                user::inbox,
                user::post_inbox,
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
                user_profile,
            ],
        )
        .mount("/api/v2", routes![api::instance::instance])
        .mount(
            "/api/v1",
            routes![
                api::status::new_status,
                api::user::new_follow,
                api::apps::new_app,
                api::preferences::preferences,
                api::user::verify_credentials,
                custom::finger_account,
                home_timeline
            ],
        )
}
