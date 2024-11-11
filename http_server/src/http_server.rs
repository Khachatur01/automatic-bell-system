use esp_idf_svc::http::Method;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::io::{EspIOError, Write};

pub struct HttpServer<'a> {
    server: EspHttpServer<'a>
}

impl<'a> HttpServer<'a> {
    pub fn new() -> Result<Self, EspIOError> {
        let mut server: EspHttpServer = EspHttpServer::new(&Configuration::default())?;

        server.fn_handler("/", Method::Get, |request| -> Result<(), EspIOError> {
            let html = "Hello World\n";
            let mut response = request.into_ok_response()?;
            response.write_all(html.as_bytes())?;

            Ok(())
        })?;

        Ok(Self { server })
    }
}
