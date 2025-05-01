use crate::Db;
use askama::Template;
use tracing::error;

use rocket::{
    FromForm,
    form::Form,
    get, post,
    response::status::BadRequest,
    response::content::RawHtml,
    serde::{Deserialize, Serialize, json::Json},
};

use rocket_db_pools::Connection;

struct AuthorizeClient {
    id: String
}

#[derive(Template)]
#[template(path = "authorize.html")]
struct AuthorizeTemplate {
    client: AuthorizeClient,
    scopes: Vec<String>,
    scope_raw: String,
    redirect_uri: String,
    user_id: String
}

#[post("/oauth/accept?<id>&<client_id>&<scope>")]
pub async fn accept(
    mut db: Connection<Db>,
    id: &str,
    client_id: &str,
    scope: &str
) -> RawHtml<String> {
    let user_id = id;
    let code = main::gen_token(15);

    // This will act as a token for the user, but we will in future say that it expires very shortly
    // and can only be used for obtaining an access token etc
    sqlx::query!(
        r#"
        INSERT INTO auth (token, user_id)
        VALUES (?1, ?2)
      "#,
        code,
        user_id
    )
    .execute(&mut **db)
    .await
    .unwrap();

    let id_token = main::gen_token(10);

    // Add an oauth entry for the `code` which /oauth/token will rewrite
    sqlx::query!(
        r#"
      INSERT INTO oauth (id_token, client_id, expires_in, scope, access_token)
      VALUES (?1, ?2, ?3, ?4, ?5)
    "#,
        id_token,
        client_id,
        3600,
        scope,
        code
    )
    .execute(&mut **db)
    .await
    .unwrap();

    // HACK: Until we are storing oauth stuff more properly we will hardcode phanpy
    RawHtml(format!(r#"
       <script>window.location.href="{}{}"</script>
    "#, "https://phanpy.social?code=", code))
}

#[get("/oauth/authorize?<client_id>&<scope>&<redirect_uri>&<response_type>")]
pub async fn authorize(
    client_id: &str,
    scope: &str,
    redirect_uri: &str,
    response_type: &str
) -> Result<RawHtml<String>, BadRequest<String>> {
    if response_type != "code" {
        error!("unknown response type {}", response_type);
        return Err(
            BadRequest(format!("unknown response type {}", response_type))
        )
    }
    
    let tmpl = AuthorizeTemplate {
        client: AuthorizeClient {
            id: client_id.to_string()
        },
        scope_raw: scope.to_string(),
        scopes: scope.split(" ").map(|s| s.to_string()).collect(),
        redirect_uri: redirect_uri.to_string(),
        user_id: "9b9d497b-2731-435f-a929-e609ca69dac9".to_string()
    };

    Ok(RawHtml(tmpl.render().unwrap()))
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub scope: String,
    pub id_token: String,
}

#[derive(Deserialize, Debug, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct NewTokenRequest {
    // pub client_id: String,
    // pub redirect_uri: String,
    // pub grant_type: String,
    pub code: String,
    // pub client_secret: String,
}

#[post("/oauth/token", data = "<req>")]
pub async fn new_token(req: Form<NewTokenRequest>, mut db: Connection<Db>) -> Json<Token> {
    let oauth = sqlx::query!(
        "
      SELECT o.*, a.*
      FROM oauth o
      INNER JOIN auth a ON a.token = ?2
      WHERE o.access_token = ?1
    ",
        req.code,
        req.code
    )
    .fetch_one(&mut **db)
    .await
    .unwrap();

    let access_token = main::gen_token(15);

    // Important: setup 'auth' first
    sqlx::query!(
        r#"
      INSERT INTO auth (token, user_id)
      VALUES (?1, ?2)
    "#,
        access_token,
        oauth.user_id
    )
    .execute(&mut **db)
    .await
    .unwrap();

    sqlx::query!(
        "UPDATE oauth SET access_token = ?1 WHERE access_token = ?2",
        access_token,
        req.code
    )
    .execute(&mut **db)
    .await
    .unwrap();

    sqlx::query!("DELETE FROM auth WHERE token = ?1", req.code)
        .execute(&mut **db)
        .await
        .unwrap();

    Json(Token {
        access_token: access_token.to_string(),
        token_type: "Bearer".to_string(),
        expires_in: oauth.expires_in,
        scope: oauth.scope.to_string(),
        id_token: oauth.id_token,
    })
}
