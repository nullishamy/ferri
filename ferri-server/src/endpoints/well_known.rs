use main::ap;
use rocket::{get, serde::json::Json};
use rocket_db_pools::Connection;

use crate::{
    Db,
    types::webfinger::{Link, WebfingerResponse},
};

#[get("/.well-known/host-meta")]
pub async fn host_meta() -> &'static str {
    r#"
      <?xml version="1.0" encoding="UTF-8"?>
      <XRD xmlns="http://docs.oasis-open.org/ns/xri/xrd-1.0">
        <Link rel="lrdd" template="https://ferri.amy.mov/.well-known/webfinger?resource={uri}"/>
      </XRD>
    "#
}

// https://mastodon.social/.well-known/webfinger?resource=acct:gargron@mastodon.social
#[get("/.well-known/webfinger?<resource>")]
pub async fn webfinger(mut db: Connection<Db>, resource: &str) -> Json<WebfingerResponse> {
    println!("Webfinger request for {}", resource);
    let acct = resource.strip_prefix("acct:").unwrap();
    let (user, _) = acct.split_once("@").unwrap();
    let user = ap::User::from_username(user, &mut **db).await;

    Json(WebfingerResponse {
        subject: resource.to_string(),
        aliases: vec![
            format!("https://ferri.amy.mov/users/{}", user.id()),
            format!("https://ferri.amy.mov/@{}", user.username()),
        ],
        links: vec![
            Link {
                rel: "http://webfinger.net/rel/profile-page".to_string(),
                ty: Some("text/html".to_string()),
                href: Some(format!("https://ferri.amy.mov/@{}", user.username())),
            },
            Link {
                rel: "self".to_string(),
                ty: Some("application/activity+json".to_string()),
                href: Some(format!("https://ferri.amy.mov/users/{}", user.id())),
            },
        ],
    })
}
