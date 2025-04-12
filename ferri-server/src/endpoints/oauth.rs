use rocket::{get, post, response::Redirect, serde::{json::Json, Deserialize, Serialize}};

#[get("/oauth/authorize?<client_id>&<scope>&<redirect_uri>&<response_type>")]
pub async fn authorize(
    client_id: &str,
    scope: &str,
    redirect_uri: &str,
    response_type: &str,
) -> Redirect {
    Redirect::temporary(format!(
        "{}?code=code-for-{}&state=state-for-{}",
        redirect_uri, client_id, client_id
    ))
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

#[post("/oauth/token")]
pub async fn new_token() -> Json<Token> {
    Json(Token {
        access_token: "9b9d497b-2731-435f-a929-e609ca69dac9".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        scope: "read write follow push".to_string(),
        id_token: "id-token".to_string(),
    })
}
