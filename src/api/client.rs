use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::de::DeserializeOwned;

use super::model::auth::Authentication;
use super::routing::Endpoint;

pub struct Api {
    pub client: Client,
    pub server: reqwest::Url,
    pub client_id: String,
    pub client_secret: String,
}

struct RequestInfo<'a> {
    headers: Option<HeaderMap>,
    route: Endpoint<'a>,
}

impl Api {
    pub fn new(server: reqwest::Url, client_id: String, client_secret: String) -> Self {
        let client = Client::builder()
            .build()
            .expect("Cannot build reqwest::Client");

        Api {
            client,
            server,
            client_id,
            client_secret,
        }
    }

    async fn fire<T: DeserializeOwned>(
        &self,
        request_info: &RequestInfo<'_>,
    ) -> Result<T, reqwest::Error> {
        let client = &self.client;
        let mut req = request_info.route.to_request(&self.server);

        if let Some(headers) = &request_info.headers {
            *req.headers_mut() = headers.clone();
        }

        client
            .execute(req)
            .await?
            .error_for_status()?
            .json::<T>()
            .await
    }

    pub async fn authenticate(
        &self,
        discord_id_hash: &str,
    ) -> Result<Authentication, reqwest::Error> {
        let route = Endpoint::Authenticate {
            id: &self.client_id,
            secret: &self.client_secret,
            discord_id_hash,
        };
        self.fire(&RequestInfo {
            headers: None,
            route,
        })
        .await
    }
}
