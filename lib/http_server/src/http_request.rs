use esp_idf_svc::http::server::{Connection, Request, Response};
use esp_idf_svc::io::Write;

fn status_response<'a, C>(mut request: Request<C>,
                          status: u16,
                          data: &[u8],
                          message: &'a str,
                          headers: &'a [(&'a str, &'a str)],) -> Result<(), C::Error>
where C: Connection {
    let mut response: Response<C> = request.into_response(status, Some(message), headers)?;
    response.write_all(data)?;
    Ok(())
}

pub trait HttpRequest<C>
where C: Connection {
    fn ok(self, data: Vec<u8>) -> Result<(), C::Error>;

    fn bad_request(self, data: Vec<u8>) -> Result<(), C::Error>;

    fn not_found(self, data: Vec<u8>) -> Result<(), C::Error>;

    fn internal_server_error(self, data: Vec<u8>) -> Result<(), C::Error>;

    fn read_all(&mut self) -> Result<Vec<u8>, C::Error>;
}

impl<C> HttpRequest<C> for Request<C>
where C: Connection {
    fn ok(self, data: Vec<u8>) -> Result<(), C::Error> {
        status_response(self, 200, &data, "Ok", &[])
    }

    fn bad_request(self, data: Vec<u8>) -> Result<(), C::Error> {
        status_response(self, 400, &data, "Bad Request", &[])
    }

    fn not_found(self, data: Vec<u8>) -> Result<(), C::Error> {
        status_response(self, 404, &data, "Not found", &[])
    }

    fn internal_server_error(self, data: Vec<u8>) -> Result<(), C::Error> {
        status_response(self, 500, &data, "Internal Server Error", &[])
    }

    fn read_all(&mut self) -> Result<Vec<u8>, C::Error> {
        let content_length: usize = self
            .header("Content-Length")
            .and_then(|content_length| content_length.parse().ok())
            .unwrap_or(0);
        
        let mut buffer: Vec<u8> = vec![0; content_length];
        self.read(buffer.as_mut_slice())?;

        Ok(buffer)
    }
}
