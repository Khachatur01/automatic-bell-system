use crate::model::auth::login_credentials::LoginCredentials;
use crate::security::error::SecurityError;
use crate::security::SecurityContext;
use esp_idf_svc::http::server::{EspHttpConnection, Request};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::sys::EspError;
use http_server::http_request::{IntoResponse, ReadData, RequestError, RequestResult};
use http_server::http_server::HttpServer;
use crate::model::auth::access_point_credentials::AccessPointCredentials;
use crate::model::auth::api_credentials::ApiCredentials;

pub fn serve(http_server: &mut HttpServer) -> Result<(), EspError> {
    http_server.add_handler(
        "/api/v1/login", Method::Post,
        move |request| login(request)
    )?;
    http_server.add_handler(
        "/api/v1/user/password", Method::Patch,
        move |request| change_user_password(request)
    )?;
    http_server.add_handler(
        "/api/v1/access-point/password", Method::Patch,
        move |request| change_access_point_password(request)
    )?;

    Ok(())
}

fn login(mut request: Request<&mut EspHttpConnection>) -> RequestResult<(), EspIOError> {
    let security_context: &SecurityContext = SecurityContext::get().map_err(RequestError::EspError)?;

    let LoginCredentials { password }: LoginCredentials = request.body()?;

    match security_context.get_access_token("", &password) {
        Ok(access_token) => request.ok(&access_token),
        Err(error) =>
            match error {
                SecurityError::EspError(error) => Err(RequestError::EspError(error)),
                SecurityError::WrongCredentials => request.forbidden(&"Unable to get access token. Wrong password."),
                SecurityError::ReadLockError => request.internal_server_error(&"Can't lock security context for reading token.."),
                SecurityError::WriteLockError => request.internal_server_error(&"Can't lock security context for writing token.."),
            }
    }
}

fn change_user_password(mut request: Request<&mut EspHttpConnection>) -> RequestResult<(), EspIOError> {
    let security_context: &SecurityContext = SecurityContext::get().map_err(RequestError::EspError)?;

    let ApiCredentials { password } = request.body()?;

    security_context.set_api_password(password.as_str()).map_err(RequestError::EspError)?;

    request.ok(&"Api password changed.")
}

fn change_access_point_password(mut request: Request<&mut EspHttpConnection>) -> RequestResult<(), EspIOError> {
    let security_context: &SecurityContext = SecurityContext::get().map_err(RequestError::EspError)?;

    let AccessPointCredentials { password } = request.body()?;

    if password.len() < 8 {
        return request.bad_request(&"Password should be minimum 8 characters.");
    }

    security_context.set_access_point_password(password.as_str()).map_err(RequestError::EspError)?;

    request.ok(&"Access point password changed.")
}
