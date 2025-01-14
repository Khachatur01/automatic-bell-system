use crate::model::auth::login_credentials::LoginCredentials;
use crate::security::error::SecurityError;
use crate::security::SecurityContext;
use esp_idf_svc::http::server::{EspHttpConnection, Request};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::sys::EspError;
use http_server::http_request::{IntoResponse, ReadData, RequestError, RequestResult};
use http_server::http_server::HttpServer;

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
    let security_context: &SecurityContext = SecurityContext::new().map_err(RequestError::EspError)?;

    let LoginCredentials { password }: LoginCredentials = request.read_body()?;

    match security_context.get_access_token("", &password) {
        Ok(access_token) => request.ok(&access_token),
        Err(error) =>
            match error {
                SecurityError::EspError(error) => Err(RequestError::EspError(error)),
                SecurityError::WrongCredentials => request.forbidden(&"Unable to get access token. Wrong password."),
            }
    }
}

fn change_user_password(mut request: Request<&mut EspHttpConnection>) -> RequestResult<(), EspIOError> {
    todo!()
}

fn change_access_point_password(mut request: Request<&mut EspHttpConnection>) -> RequestResult<(), EspIOError> {
    todo!()
}
