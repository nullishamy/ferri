use crate::Db;
use main::types::{api, get};
use rocket::{get, serde::json::Json, State};
use rocket_db_pools::Connection;
use tracing::info;

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
pub async fn webfinger(
    mut db: Connection<Db>,
    helpers: &State<crate::Helpers>,
    resource: &str,
) -> Json<api::WebfingerHit> {
    let config = &helpers.config;
    info!(?resource, "incoming webfinger request");

    let acct = resource.strip_prefix("acct:").unwrap();
    let (user, _) = acct.split_once("@").unwrap();
    let user = get::user_by_username(user, &mut **db)
        .await
        .unwrap();

    Json(api::WebfingerHit {
        subject: resource.to_string(),
        aliases: vec![
            config.user_url(&user.id.0),
            config.user_web_url(&user.username),
        ],
        links: vec![
            api::WebfingerLink {
                rel: "http://webfinger.net/rel/profile-page".to_string(),
                ty: Some("text/html".to_string()),
                href: Some(config.user_web_url(&user.username)),
            },
            api::WebfingerLink {
                rel: "self".to_string(),
                ty: Some("application/activity+json".to_string()),
                href: Some(config.user_url(&user.id.0)),
            },
        ],
    })
}
