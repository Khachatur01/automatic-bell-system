use crate::security::error::SecurityError;
use crate::security::SecurityContext;
use esp_idf_svc::http::server::{EspHttpConnection, Request};
use esp_idf_svc::io::EspIOError;
use http_server::http_request::{RequestError, RequestResult};

pub fn authenticate_request(request: &Request<&mut EspHttpConnection>) -> RequestResult<(), EspIOError> {
    let access_token = request.header("Access-Token");

    match access_token {
        Some(access_token) => {
            let is_valid_access_token: bool = SecurityContext::get()
                .map_err(RequestError::EspError)?
                .is_valid_access_token_token(access_token)
                .map_err(|error: SecurityError| RequestError::Security(error.to_string()))?;

            if !is_valid_access_token {
                Err(RequestError::Security("Invalid access token!".to_string()))
            } else {
                Ok(())
            }
        },
        None => Err(RequestError::Security("Missing Access-Token header.".to_string()))
    }
}
