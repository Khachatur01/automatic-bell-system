use serde::Deserialize;

#[derive(Deserialize)]
pub struct ApiCredentials {
    pub password: String,
}
