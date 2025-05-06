use crate::types::ap;
use std::fmt::Debug;
use serde::Serialize;
use thiserror::Error;
use tracing::{Level, error, event, info};

use reqwest::{IntoUrl, Response};
use url::Url;

use rsa::{
    RsaPrivateKey,
    pkcs1v15::SigningKey,
    pkcs8::DecodePrivateKey,
    sha2::{Digest, Sha256},
    signature::{RandomizedSigner, SignatureEncoding},
};

use base64::prelude::*;
use chrono::Utc;

use super::outbox::PreparedActivity;

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

    pub async fn get_note(&self, url: &str) -> Result<ap::Post, HttpError> {
        self.get("Note", url).await
    }

    pub async fn post_activity<T : Serialize + Debug>(
        &self,
        inbox: &str,
        activity: PreparedActivity<T>
    ) -> Result<String, HttpError> {
        let http_result = self
            .client
            .post(inbox)
            .activity()
            .json(activity)
            .sign(self.key_id)
            .send()
            .await;
        
        if let Err(e) = http_result {
            error!("could not load url {}: {:#?}", inbox, e);
            return Err(HttpError::LoadFailure("Activity".to_string(), inbox.to_string()));
        }

        let raw_body = http_result.unwrap().text().await;
        if let Err(e) = raw_body {
            error!("could not get text for url {}: {:#?}", inbox, e);
            return Err(HttpError::LoadFailure("Activity".to_string(), inbox.to_string()));
        }

        let raw_body = raw_body.unwrap();
        Ok(raw_body.to_string())
    }
}

pub struct HttpClient {
    client: reqwest::Client,
}

#[derive(Debug)]
pub struct PostSignature {
    date: String,
    digest: String,
    signature: String,
}

#[derive(Debug)]
struct GetSignature {
    date: String,
    signature: String,
}

enum RequestVerb {
    GET,
    POST,
}

pub struct RequestBuilder {
    verb: RequestVerb,
    url: Url,
    body: String,
    inner: reqwest::RequestBuilder,
}

impl RequestBuilder {
    pub fn json(mut self, json: impl Serialize + Sized) -> RequestBuilder {
        let body = serde_json::to_string(&json).unwrap();
        self.inner = self.inner.body(body.clone());
        self.body = body;
        self
    }

    pub fn activity(mut self) -> RequestBuilder {
        self.inner = self
            .inner
            .header("Content-Type", "application/activity+json")
            .header("Accept", "application/activity+json");
        self
    }

    pub async fn send(self) -> Result<Response, reqwest::Error> {
        event!(Level::DEBUG, ?self.inner, "sending an http request");

        self.inner.send().await
    }

    pub fn sign(mut self, key_id: &str) -> RequestBuilder {
        match self.verb {
            RequestVerb::GET => {
                let sig = self.sign_get_request(key_id);
                self.inner = self
                    .inner
                    .header("Date", sig.date)
                    .header("Signature", sig.signature);
                self
            }
            RequestVerb::POST => {
                let sig = self.sign_post_request(key_id);
                self.inner = self
                    .inner
                    .header("Date", sig.date)
                    .header("Digest", sig.digest)
                    .header("Signature", sig.signature);
                self
            }
        }
    }

    fn sign_get_request(&self, key_id: &str) -> GetSignature {
        let url = &self.url;
        let host = url.host_str().unwrap();
        let path = url.path();

        let private_key =
            RsaPrivateKey::from_pkcs8_pem(include_str!("../../../private.pem")).unwrap();
        let signing_key = SigningKey::<Sha256>::new(private_key);

        // UTC=GMT for our purposes, use it
        // RFC7231 is hardcoded to use GMT for.. some reason
        let ts = Utc::now();

        // RFC7231 string
        let date = ts.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let to_sign = format!(
            "(request-target): get {}\nhost: {}\ndate: {}",
            path, host, date
        );

        let signature = signing_key.sign_with_rng(&mut rand::rngs::OsRng, &to_sign.into_bytes());
        let header = format!(
            "keyId=\"{}\",algorithm=\"rsa-sha256\",headers=\"(request-target) host date\",signature=\"{}\"",
            key_id,
            BASE64_STANDARD.encode(signature.to_bytes())
        );

        GetSignature {
            date,
            signature: header,
        }
    }

    fn sign_post_request(&self, key_id: &str) -> PostSignature {
        let body = &self.body;
        let url = &self.url;

        let host = url.host_str().unwrap();
        let path = url.path();

        let private_key =
            RsaPrivateKey::from_pkcs8_pem(include_str!("../../../private.pem")).unwrap();
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

        let to_sign = format!(
            "(request-target): post {}\nhost: {}\ndate: {}\ndigest: {}",
            path, host, date, digest
        );

        let signature = signing_key.sign_with_rng(&mut rand::rngs::OsRng, &to_sign.into_bytes());
        let header = format!(
            "keyId=\"{}\",algorithm=\"rsa-sha256\",headers=\"(request-target) host date digest\",signature=\"{}\"",
            key_id,
            BASE64_STANDARD.encode(signature.to_bytes())
        );

        PostSignature {
            date,
            digest,
            signature: header,
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn get(&self, url: impl IntoUrl + Clone) -> RequestBuilder {
        RequestBuilder {
            verb: RequestVerb::GET,
            url: url.clone().into_url().unwrap(),
            body: String::new(),
            inner: self.client.get(url),
        }
    }

    pub fn post(&self, url: impl IntoUrl + Clone) -> RequestBuilder {
        RequestBuilder {
            verb: RequestVerb::POST,
            url: url.clone().into_url().unwrap(),
            body: String::new(),
            inner: self.client.post(url),
        }
    }
}
