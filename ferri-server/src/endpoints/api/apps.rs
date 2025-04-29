use rocket::{form::Form, post, serde::json::Json};

use crate::Db;
use main::types::api;
use rocket::FromForm;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, FromForm, Clone)]
pub struct OauthApp {
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub scopes: String,
}

#[post("/apps", data = "<app>")]
pub async fn new_app(
    app: Form<OauthApp>,
    mut db: Connection<Db>,
) -> Json<api::CredentialApplication> {
    let secret = main::gen_token(15);

    // Abort when we encounter a duplicate
    let is_app_present = sqlx::query!(
        r#"
      INSERT INTO app (client_id, client_secret, scopes)
      VALUES (?1, ?2, ?3)
    "#,
        app.client_name,
        app.scopes,
        secret
    )
    .execute(&mut **db)
    .await
    .is_err();

    let mut app: OauthApp = app.clone();

    if is_app_present {
        let existing_app = sqlx::query!("SELECT * FROM app WHERE client_id = ?1", app.client_name)
            .fetch_one(&mut **db)
            .await
            .unwrap();

        app.client_name = existing_app.client_id;
        app.scopes = existing_app.scopes;
    }

    Json(api::CredentialApplication {
        name: app.client_name.clone(),
        scopes: app.scopes.clone(),
        redirect_uris: app.redirect_uris.clone(),
        client_id: app.client_name.clone(),
        client_secret: secret,
    })
}
