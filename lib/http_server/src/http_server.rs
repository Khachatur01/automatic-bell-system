use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer, Request};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::sys::{EspError};
use crate::http_request::{IntoResponse, RequestError, RequestResult};

pub struct HttpServer<'a> {
    server: EspHttpServer<'a>,
}

impl<'a> HttpServer<'a> {
    pub fn new() -> Result<Self, EspIOError> {
        let mut configuration: Configuration = Configuration::default();
        configuration.uri_match_wildcard = true;
        configuration.stack_size = 8 * 1024;

        let mut server: EspHttpServer = EspHttpServer::new(&configuration)?;

        server.fn_handler::<RequestError<EspIOError>, _>("/*?", Method::Options, move |esp_http_request: Request<&mut EspHttpConnection>| -> RequestResult<(), EspIOError> {
            println!("Options");
            esp_http_request.ok(&"Returning OK response on OPTIONS request").map(|_| ())
        })?;

        Ok(Self { server })
    }

    pub fn add_handler<F>(
        &mut self,
        uri: &str,
        method: Method,
        handle_request: F,
    ) -> Result<(), EspError>
    where
        F: for<'r> Fn(Request<&mut EspHttpConnection<'r>>) -> RequestResult<(), EspIOError> + Send + 'static,
    {
        self.server.fn_handler::<RequestError<EspIOError>, _>(uri, method, move |esp_http_request: Request<&mut EspHttpConnection>| -> RequestResult<(), EspIOError> {
            handle_request(esp_http_request).map(|_| ())
        })?;

        Ok(())
    }
}
