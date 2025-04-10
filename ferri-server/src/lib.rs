use rocket::serde::json::Json;
use rocket::serde::json::serde_json;
use rocket::{Rocket, Build, build, get, post, routes, http::{MediaType, ContentType}};
use reqwest;

use uuid::Uuid;

use base64::prelude::*;

use rocket::serde::Serialize;
use rocket::serde::Deserialize;

use rocket::Request;
use rocket::request::Outcome;
use rocket::request::FromRequest;
use rocket::FromForm;

use rsa::{RsaPrivateKey, pkcs8::DecodePrivateKey};
use rsa::pkcs1v15::SigningKey;
use rsa::signature::{RandomizedSigner, SignatureEncoding};
use rsa::sha2::{Digest, Sha256};

use rocket::form::Form;

use url::Url;
use chrono::Utc;

mod ap;

use rocket_db_pools::{Database, Connection};
use rocket_db_pools::sqlx::{self, Row};

#[derive(Database)]
#[database("sqlite_ferri")]
struct Db(sqlx::SqlitePool);

#[get("/users/<user>/inbox")]
async fn inbox(user: String) -> Json<ap::OrderedCollection> {
    dbg!(&user);
    Json(ap::OrderedCollection {
        ty: "OrderedCollection".to_string(),
        summary: format!("Inbox for {}", user),
        total_items: 0,
        ordered_items: vec![]
    })
}

#[post("/users/<user>/inbox", data="<body>")]
async fn post_inbox(mut db: Connection<Db>, user: String, body: String) {
    let client = reqwest::Client::new();
    
    let min = serde_json::from_str::<ap::MinimalActivity>(&body).unwrap();
    match min.ty.as_str() {
        "Delete" => {
            let activity = serde_json::from_str::<ap::DeleteActivity>(&body);
            dbg!(activity);
        }
        "Follow" => {
            let activity = serde_json::from_str::<ap::FollowActivity>(&body).unwrap();
            dbg!(&activity);
             let user = client.get(&activity.actor)
                .header("Accept", "application/activity+json")
                .send()
                .await.unwrap()
                .json::<ap::Person>()
                .await.unwrap();
            
            sqlx::query!(r#"
              INSERT INTO actor (id, inbox, outbox)
              VALUES ( ?1, ?2, ?3 )
              ON CONFLICT(id) DO NOTHING;
            "#, activity.actor, user.inbox, user.outbox)
                .execute(&mut **db)
                .await.unwrap();

            sqlx::query!(r#"
              INSERT INTO follow (id, follower_id, followed_id)
              VALUES ( ?1, ?2, ?3 )
              ON CONFLICT(id) DO NOTHING;
            "#, activity.id, activity.actor, activity.object)
                .execute(&mut **db)
                .await.unwrap();
            
            let accept = ap::AcceptActivity {
                ty: "Accept".to_string(),
                actor: "https://ferri.amy.mov/users/amy".to_string(),
                object: activity.id
            };

            let key_id = "https://ferri.amy.mov/users/amy#main-key".to_string();
            let document = serde_json::to_string(&accept).unwrap();
            let signature = sign_post_request(key_id, user.inbox.clone(), document);
            dbg!(&signature);
            
            let follow_res = client.post(user.inbox)
                .header("Content-Type", "application/activity+json")
                .header("Date", signature.date)
                .header("Digest", signature.digest)
                .header("Signature", signature.signature)
                .json(&accept)
                .send()
                .await.unwrap()
                .text()
                .await.unwrap();

            dbg!(follow_res);
        }
        unknown => {
            eprintln!("WARN: Unknown activity '{}' - {}", unknown, body);
        }
    }
    
    dbg!(min);
    println!("body: {}", body);
}

#[get("/users/<user>/outbox")]
async fn outbox(user: String) -> Json<ap::OrderedCollection> {
    dbg!(&user);
    Json(ap::OrderedCollection {
        ty: "OrderedCollection".to_string(),
        summary: format!("Outbox for {}", user),
        total_items: 0,
        ordered_items: vec![]
    })
}

#[get("/users/<user>/followers")]
async fn followers(mut db: Connection<Db>, user: String) -> Json<ap::OrderedCollection> {
    let target = FerriUser::by_name(&user, &mut **db).await;

    let followers = sqlx::query!( r#"
      SELECT follower_id FROM follow
      WHERE followed_id = ?
    "#, target.actor_id)
       .fetch_all(&mut **db)
       .await.unwrap();
    
    Json(ap::OrderedCollection {
        ty: "OrderedCollection".to_string(),
        summary: format!("Followers for {}", user),
        total_items: 1,
        ordered_items: followers.into_iter().map(|f| f.follower_id).collect::<Vec<_>>()
    })
}

#[derive(Debug)]
struct FerriUser {
    id: String,
    actor_id: String,
    display_name: String
}

impl FerriUser {
    async fn by_name<'a>(
        name: &'a str,
        conn: impl sqlx::Executor<'a, Database = sqlx::Sqlite>
    ) -> FerriUser {
        sqlx::query_as!(FerriUser, r#"
      SELECT * FROM user
      WHERE display_name = ?
    "#, name)
        .fetch_one(conn)
        .await.unwrap()
    }    
}


#[get("/users/<user>/following")]
async fn following(mut db: Connection<Db>, user: String) -> Json<ap::OrderedCollection> {
    let target = FerriUser::by_name(&user, &mut **db).await;

    let following = sqlx::query!( r#"
      SELECT followed_id FROM follow
      WHERE follower_id = ?
    "#, target.actor_id)
       .fetch_all(&mut **db)
       .await.unwrap();
    
    Json(ap::OrderedCollection {
        ty: "OrderedCollection".to_string(),
        summary: format!("Following for {}", user),
        total_items: 1,
        ordered_items: following.into_iter().map(|f| f.followed_id).collect::<Vec<_>>()
    })
}

fn activity_type() -> ContentType {
    ContentType(MediaType::new("application", "activity+json"))
}

#[get("/users/<user>/posts/<post>")]
async fn get_post(user: String, post: String) -> (ContentType, Json<ap::Post>) {
    (activity_type(), Json(ap::Post {
        id: format!("https://ferri.amy.mov/users/{}/posts/{}", user, post),
        context: "https://www.w3.org/ns/activitystreams".to_string(),
        ty: "Note".to_string(),
        content: "My first post".to_string(),
        ts: "2025-04-10T10:48:11Z".to_string(),
        to: vec!["https://ferri.amy.mov/users/amy/followers".to_string()],
        cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
    }))
}

#[get("/users/<user>")]
async fn user(user: String) -> (ContentType, Json<ap::Person>) {
    (activity_type(), Json(ap::Person {
        context: "https://www.w3.org/ns/activitystreams".to_string(),
        ty:  "Person".to_string(),
        id:  format!("https://ferri.amy.mov/users/{}", user),
        name:  user.clone(),
        preferred_username: user.clone(),
        followers: format!("https://ferri.amy.mov/users/{}/followers", user),
        following: format!("https://ferri.amy.mov/users/{}/following", user),
        summary: format!("ferri {}", user),
        inbox:  format!("https://ferri.amy.mov/users/{}/inbox", user),
        outbox:  format!("https://ferri.amy.mov/users/{}/outbox", user),
        public_key: Some(ap::UserKey {
            id: format!("https://ferri.amy.mov/users/{}#main-key", user),
            owner: format!("https://ferri.amy.mov/users/{}", user),
            public_key: include_str!("../../public.pem").to_string(),
        })
    }))
}

#[get("/")]
async fn user_profile() -> (ContentType, &'static str) {
    (ContentType::HTML, "<p>hello</p>")
}

#[get("/activities/<activity>")]
async fn activity(activity: String) {
    dbg!(activity);
}

// https://mastodon.social/.well-known/webfinger?resource=acct:gargron@mastodon.social
#[get("/.well-known/webfinger?<resource>")]
async fn webfinger(mut db: Connection<Db>, resource: &str) -> Json<ap::WebfingerResponse> {
    println!("Webfinger request for {}", resource);
    let acct = resource.strip_prefix("acct:").unwrap();
    let (user, _) = acct.split_once("@").unwrap();

    let user = FerriUser::by_name(user, &mut **db).await;
    dbg!(&user);
    
    Json(ap::WebfingerResponse {
        subject: resource.to_string(),
        aliases: vec![
            format!("https://ferri.amy.mov/users/{}", user.id),
            format!("https://ferri.amy.mov/@{}", user.id)
        ],
        links: vec![
            ap::Link {
                rel: "http://webfinger.net/rel/profile-page".to_string(),
                ty: Some("text/html".to_string()),
                href: Some(format!("https://ferri.amy.mov/@{}", user.id))
            },
            ap::Link {
                rel: "self".to_string(),
                ty: Some("application/activity+json".to_string()),
                href: Some(format!("https://ferri.amy.mov/users/{}", user.id))
            }
        ]
    })
}

async fn resolve_user(acct: &str, host: &str) -> ap::Person {
    let client = reqwest::Client::new();
    let url = format!("https://{}/.well-known/webfinger?resource=acct:{}", host, acct);
    let wf = client.get(url)
        .send()
        .await.unwrap()
        .json::<ap::WebfingerResponse>()
        .await.unwrap();

    let actor_link = wf.links
        .iter()
        .find(|l| l.ty == Some("application/activity+json".to_string()))
        .unwrap();

    let href = actor_link.href.as_ref().unwrap();
    client.get(href)
        .header("Accept", "application/activity+json")
        .send()
        .await.unwrap()
        .json::<ap::Person>()
        .await.unwrap()
}

#[derive(Debug)]
struct PostSignature {
    date: String,
    digest: String,
    signature: String    
}

#[derive(Debug)]
struct GetSignature {
    date: String,
    signature: String    
}

fn sign_get_request(key_id: String, raw_url: String) -> GetSignature {
    let url = Url::parse(&raw_url).unwrap();

    let host = url.host_str().unwrap();
    let path = url.path();
    
    let private_key = RsaPrivateKey::from_pkcs8_pem(include_str!("../../private.pem")).unwrap();
    let signing_key = SigningKey::<Sha256>::new(private_key);

    // UTC=GMT for our purposes, use it
    // RFC7231 is hardcoded to use GMT for.. some reason
    let ts = Utc::now();
    
    // RFC7231 string
    let date = ts.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    dbg!(&date);
    
    let to_sign = format!("(request-target): get {}\nhost: {}\ndate: {}",
                          path,
                          host,
                          date);

    let signature = signing_key.sign_with_rng(&mut rand::rngs::OsRng, &to_sign.into_bytes());
    let header = format!(
        "keyId=\"{}\",algorithm=\"rsa-sha256\",headers=\"(request-target) host date\",signature=\"{}\"",
        key_id,
        BASE64_STANDARD.encode(signature.to_bytes())
    );

    GetSignature {
        date: date,
        signature: header
    }
}

fn sign_post_request(key_id: String, raw_url: String, body: String) -> PostSignature {
    let url = Url::parse(&raw_url).unwrap();

    let host = url.host_str().unwrap();
    let path = url.path();
    
    let private_key = RsaPrivateKey::from_pkcs8_pem(include_str!("../../private.pem")).unwrap();
    let signing_key = SigningKey::<Sha256>::new(private_key);

    let mut hasher = Sha256::new();
    hasher.update(body);
    let sha256 = hasher.finalize();

    let b64 = BASE64_STANDARD.encode(sha256);
    let digest = format!("SHA-256={}", b64);

    // UTC=GMT for our purposes, use it
    // RFC7231 is hardcoded to use GMT for.. some reason
    let ts = Utc::now();
    
    // RFC7231 string
    let date = ts.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    dbg!(&date);
    
    let to_sign = format!("(request-target): post {}\nhost: {}\ndate: {}\ndigest: {}",
                          path,
                          host,
                          date,
                          digest);

    let signature = signing_key.sign_with_rng(&mut rand::rngs::OsRng, &to_sign.into_bytes());
    let header = format!(
        "keyId=\"{}\",algorithm=\"rsa-sha256\",headers=\"(request-target) host date digest\",signature=\"{}\"",
        key_id,
        BASE64_STANDARD.encode(signature.to_bytes())
    );

    PostSignature {
        date: date,
        digest: digest,
        signature: header
    }
}

#[get("/test")]
async fn index() -> &'static str {
    let client = reqwest::Client::new();
    let user = resolve_user("amy@fedi.amy.mov", "fedi.amy.mov").await;
    dbg!(&user);

    let post = ap::CreateActivity {
        id: "https://ferri.amy.mov/activities/amy/20".to_string(),
        ty: "Create".to_string(),
        summary: "Amy create a note".to_string(),
        actor: "https://ferri.amy.mov/users/amy".to_string(),
        object: ap::Post {
            context: "https://www.w3.org/ns/activitystreams".to_string(),
            id: "https://ferri.amy.mov/users/amy/posts/20".to_string(),
            ty: "Note".to_string(),
            content: "My first post".to_string(),
            ts: "2025-04-10T10:48:11Z".to_string(),
            to: vec!["https://ferri.amy.mov/users/amy/followers".to_string()],
            cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        },
        ts: "2025-04-10T10:48:11Z".to_string(),
        to: vec!["https://ferri.amy.mov/users/amy/followers".to_string()],
        cc: vec![],
    };

    let key_id = "https://ferri.amy.mov/users/amy#main-key".to_string();
    let document = serde_json::to_string(&post).unwrap();
    let signature = sign_post_request(key_id, user.inbox.clone(), document);
    dbg!(&signature);
    
    let follow_res = client.post(user.inbox)
        .header("Content-Type", "application/activity+json")
        .header("Accept", "application/activity+json")
        .header("Date", signature.date)
        .header("Digest", signature.digest)
        .header("Signature", signature.signature)
        .json(&post)
        .send()
        .await.unwrap()
        .text()
        .await.unwrap();

    println!("{}", follow_res);

    "Hello, world!"
}

#[derive(Serialize, Deserialize, Debug, FromForm)]
#[serde(crate = "rocket::serde")]
struct Status {
    status: String,
}

#[derive(Debug)]
struct AuthenticatedUser {
    id: String
}

#[derive(Debug)]
enum LoginError {
    InvalidData,
    UsernameDoesNotExist,
    WrongPassword
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for AuthenticatedUser {
    type Error = LoginError;
    async fn from_request(request: &'a Request<'_>) -> Outcome<AuthenticatedUser, LoginError> {
        let token = request.headers().get_one("Authorization").unwrap();
        Outcome::Success(AuthenticatedUser {
            id: token.to_string()
        })
    }
}

#[post("/statuses", data="<status>")]
async fn new_status(mut db: Connection<Db>, status: Form<Status>, user: AuthenticatedUser) {
    let user = FerriUser::by_name(&user.id, &mut **db).await;
    let post_id = Uuid::new_v4();
    let uri = format!("https://ferri.amy.mov/users/amy/posts/{}", post_id);
    
    let post = sqlx::query!(r#"
      INSERT INTO post (id, user_id, content)
      VALUES (?1, ?2, ?3)
      RETURNING *
    "#, uri, user.id, status.status).fetch_one(&mut **db).await;
    
    dbg!(user, status, post);
}

pub fn launch() -> Rocket<Build> {
    build().attach(Db::init())
        .mount("/", routes![
            index,
            inbox,
            post_inbox,
            outbox,
            user,
            user_profile,
            get_post,
            followers,
            following,
            activity,
            webfinger
        ])
        .mount("/api/v1", routes![
            new_status
        ])
}
