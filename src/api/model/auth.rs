use serde::Deserialize;

#[derive(Deserialize)]
pub struct Authentication {
    pub access_token: String,
}
