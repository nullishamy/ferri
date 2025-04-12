use rocket::{form::Form, post, serde::json::Json};

use crate::types::oauth::{App, CredentialApplication};

#[post("/apps", data = "<app>")]
pub async fn new_app(app: Form<App>) -> Json<CredentialApplication> {
    Json(CredentialApplication {
        name: app.client_name.clone(),
        scopes: app.scopes.clone(),
        redirect_uris: app.redirect_uris.clone(),
        client_id: format!("id-for-{}", app.client_name),
        client_secret: format!("secret-for-{}", app.client_name),
    })
}
