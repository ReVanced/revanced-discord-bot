use poise::serenity_prelude::JsonMap;
use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::to_vec;

use super::model::api::Api;
use super::model::auth::Authentication;

pub struct ApiClient<'a> {
    pub client: Client,
    api: &'a Api,
}

struct Request<'a> {
    headers: Option<HeaderMap>,
    body: Option<&'a [u8]>,
    route: super::routing::RouteInfo,
}

impl ApiClient<'_> {
    pub fn new(api: &Api) -> Self {
        let client = Client::builder()
            .build()
            .expect("Cannot build reqwest::Client");

        ApiClient {
            client,
            api,
        }
    }

    async fn fire<T: DeserializeOwned>(&self, request: &Request<'_>) -> Result<T, reqwest::Error> {
        let client = &self.client;

        Ok(client
            .execute(client.request(request.route, url).build()?)
            .await?
            .json::<T>()
            .await
            .map_err(From::from)
            .unwrap())
    }

    pub async fn authenticate(
        &self,
        discord_id_hash: &str,
    ) -> Result<Authentication, reqwest::Error> {
        let api = &self.api;

        let body = &JsonMap::new();
        for (k, v) in [
            ("id", api.client_id),
            ("secret", api.client_secret),
            ("discord_id_hash", discord_id_hash.to_string()),
        ] {
            body.insert(k.to_string(), v)
        }

        Ok(self
            .fire(&Request {
                headers: None,
                body: Some(&to_vec(body).unwrap()),
                route: RouteInfo::Authenticate,
            })
            .await?)
    }
}
