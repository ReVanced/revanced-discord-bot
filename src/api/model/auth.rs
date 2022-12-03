use serde::Deserialize;
#[derive(Deserialize)]
pub struct Authentication<'a> {
    pub token: &'a str,
}
