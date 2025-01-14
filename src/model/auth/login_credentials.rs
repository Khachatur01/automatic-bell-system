use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginCredentials {
    pub password: String,
}
