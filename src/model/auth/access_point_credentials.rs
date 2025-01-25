use serde::Deserialize;

#[derive(Deserialize)]
pub struct AccessPointCredentials {
    pub password: String,
}
