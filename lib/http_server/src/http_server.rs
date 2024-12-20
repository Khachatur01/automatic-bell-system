use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::{EspIOError, Write};
use esp_idf_svc::sys::EspError;
use std::fmt::Debug;

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
        handler_function: F,
    ) -> Result<(), EspError>
    where
        F: Fn(HttpRequest) -> HttpResponse + Send + 'static,
    {
        self.server.fn_handler(uri, method, move |mut esp_http_request| -> Result<(), EspIOError> {
            let content_length: usize = esp_http_request
                .header("Content-Length")
                .and_then(|content_length| content_length.parse().ok())
                .unwrap_or(0);

            let mut buffer: Vec<u8> = vec![0; content_length];
            esp_http_request.read(buffer.as_mut_slice())?;

            let request = HttpRequest {
                uri: String::from(esp_http_request.uri()),
                data: buffer
            };

            let mut response: HttpResponse = handler_function(request);

            let status: u16 = response.status;
            let message: Option<&str> = response.message.as_deref();
            let headers: [(&str, &str); 0] = [];

            let mut esp_http_response = esp_http_request.into_response(status, message, &headers)?;
            esp_http_response.write_all(response.data.as_slice())?;

            Ok(())
        })?;

        Ok(())
    }
}
