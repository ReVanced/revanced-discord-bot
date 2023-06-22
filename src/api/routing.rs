use reqwest::{Body, Method, Request};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Endpoint<'a> {
    Authenticate {
        id: &'a str,
        secret: &'a str,
        discord_id_hash: &'a str,
    },
}

macro_rules! route {
    ($self:ident, $server:ident, $endpoint:literal, $method:ident) => {{
        let mut req = Request::new(Method::$method, $server.join($endpoint).unwrap());
        *req.body_mut() = Some(Body::from(serde_json::to_vec($self).unwrap()));
        req
    }};
}

impl Endpoint<'_> {
    pub fn to_request(&self, server: &reqwest::Url) -> Request {
        match self {
            Self::Authenticate {
                ..
            } => route!(self, server, "/auth/", POST),
        }
    }
}
