use crate::Db;
use rocket::{
    FromForm,
    form::Form,
    get, post,
    response::Redirect,
    serde::{Deserialize, Serialize, json::Json},
};
use rocket_db_pools::Connection;

#[get("/oauth/authorize?<client_id>&<scope>&<redirect_uri>&<response_type>")]
pub async fn authorize(
    client_id: &str,
    scope: &str,
    redirect_uri: &str,
    response_type: &str,
    mut db: Connection<Db>,
) -> Redirect {
    // For now, we will always authorize the request and assign it to an admin user
    let user_id = "9b9d497b-2731-435f-a929-e609ca69dac9";
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

    Redirect::temporary(format!("{}?code={}", redirect_uri, code))
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
    client_id: String,
    redirect_uri: String,
    grant_type: String,
    code: String,
    client_secret: String,
}

#[post("/oauth/token", data = "<req>")]
pub async fn new_token(req: Form<NewTokenRequest>, mut db: Connection<Db>) -> Json<Token> {
    let oauth = sqlx::query!("
      SELECT o.*, a.*
      FROM oauth o
      INNER JOIN auth a ON a.token = ?2
      WHERE o.access_token = ?1
    ", req.code, req.code)
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
