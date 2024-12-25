use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer, Request};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::sys::{EspError, ESP_ERR_INVALID_ARG};
use std::fmt::Debug;
use crate::http_request;
use crate::http_request::{RequestError, RequestResult};

pub struct HttpServer<'a> {
    server: EspHttpServer<'a>,
}

impl<'a> HttpServer<'a> {
    pub fn new() -> Result<Self, EspIOError> {
        let mut configuration: Configuration = Configuration::default();
        configuration.uri_match_wildcard = true;

        let mut server: EspHttpServer = EspHttpServer::new(&configuration)?;

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
        self.server.fn_handler(uri, method, move |esp_http_request: Request<&mut EspHttpConnection>| -> Result<(), EspIOError> {
            handle_request(esp_http_request).map(|_| ()).map_err(|error| {
                match error {
                    http_request::RequestError::SerdeJsonError(error) => {
                        println!("{error}");
                        let esp_error: EspError = EspError::from(ESP_ERR_INVALID_ARG).unwrap();
                        EspIOError::from(esp_error)
                    },
                    http_request::RequestError::SerdeURLError(error) => {
                        println!("{error}");
                        let esp_error: EspError = EspError::from(ESP_ERR_INVALID_ARG).unwrap();
                        EspIOError::from(esp_error)
                    },
                    http_request::RequestError::ConnectionError(error) => error,
                }
            })
        })?;

        Ok(())
    }
}
