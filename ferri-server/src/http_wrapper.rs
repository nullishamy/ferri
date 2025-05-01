use crate::http::HttpClient;
use main::types::ap;
use std::fmt::Debug;
use thiserror::Error;
use tracing::{Level, error, event, info};

pub struct HttpWrapper<'a> {
    client: &'a HttpClient,
    key_id: &'a str,
}

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("entity of type `{0}` @ URL `{1}` could not be loaded")]
    LoadFailure(String, String),
    #[error("entity of type `{0}` @ URL `{1}` could not be parsed ({2})")]
    ParseFailure(String, String, String),
}

impl<'a> HttpWrapper<'a> {
    pub fn new(client: &'a HttpClient, key_id: &'a str) -> HttpWrapper<'a> {
        Self { client, key_id }
    }

    pub fn client(&self) -> &'a HttpClient {
        self.client
    }

    async fn get<T: serde::de::DeserializeOwned + Debug>(
        &self,
        ty: &str,
        url: &str,
    ) -> Result<T, HttpError> {
        let ty = ty.to_string();
        event!(Level::INFO, url, "loading {}", ty);

        let http_result = self
            .client
            .get(url)
            .sign(self.key_id)
            .activity()
            .send()
            .await;

        if let Err(e) = http_result {
            error!("could not load url {}: {:#?}", url, e);
            return Err(HttpError::LoadFailure(ty, url.to_string()));
        }

        let raw_body = http_result.unwrap().text().await;
        if let Err(e) = raw_body {
            error!("could not get text for url {}: {:#?}", url, e);
            return Err(HttpError::LoadFailure(ty, url.to_string()));
        }

        let raw_body = raw_body.unwrap();
        info!("raw body {}", raw_body);
        let decoded = serde_json::from_str::<T>(&raw_body);

        if let Err(e) = decoded {
            error!(
                "could not parse {} for url {}: {:#?} {}",
                ty, url, e, &raw_body
            );
            return Err(HttpError::ParseFailure(ty, url.to_string(), e.to_string()));
        }

        Ok(decoded.unwrap())
    }

    pub async fn get_person(&self, url: &str) -> Result<ap::Person, HttpError> {
        self.get("Person", url).await
    }
}
