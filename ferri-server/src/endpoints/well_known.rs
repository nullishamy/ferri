use main::ap;
use rocket::{get, serde::json::Json, State};
use rocket_db_pools::Connection;
use tracing::info;

use crate::{
    Config,
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
pub async fn webfinger(mut db: Connection<Db>, config: &State<Config>, resource: &str) -> Json<WebfingerResponse> {
    info!(?resource, "incoming webfinger request");

    let acct = resource.strip_prefix("acct:").unwrap();
    let (user, _) = acct.split_once("@").unwrap();
    let user = ap::User::from_username(user, &mut **db).await;

    Json(WebfingerResponse {
        subject: resource.to_string(),
        aliases: vec![
            config.user_url(user.id()),
            config.user_web_url(user.username())
        ],
        links: vec![
            Link {
                rel: "http://webfinger.net/rel/profile-page".to_string(),
                ty: Some("text/html".to_string()),
                href: Some(config.user_web_url(user.username())),
            },
            Link {
                rel: "self".to_string(),
                ty: Some("application/activity+json".to_string()),
                href: Some(config.user_url(user.id())),
            },
        ],
    })
}
