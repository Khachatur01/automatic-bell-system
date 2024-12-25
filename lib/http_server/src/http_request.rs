use esp_idf_svc::http::server::{Connection, Request, Response};
use esp_idf_svc::io::Write;
use serde::de::{DeserializeOwned, Error as _};
use serde::{Serialize};

fn status_response<'a, C, Data>(mut request: Request<C>,
                                status: u16,
                                data: &Data,
                                message: &'a str,
                                headers: &'a [(&'a str, &'a str)],) -> RequestResult<(), C::Error>
where C: Connection,
      Data: Serialize {

    let mut response: Response<C> = request
        .into_response(status, Some(message), headers)
        .map_err(RequestError::ConnectionError)?;

    let json_data: String = serde_json::to_string(data)
        .map_err(RequestError::SerdeJsonError)?;

    let data: &[u8] = json_data.as_bytes();

    response.write_all(data)
        .map_err(RequestError::ConnectionError)?;

    Ok(())
}

#[derive(Debug)]
pub enum RequestError<ConnectionError> {
    SerdeJsonError(serde_json::Error),
    SerdeURLError(serde_urlencoded::de::Error),
    ConnectionError(ConnectionError),
}

pub type RequestResult<V, ConnectionError> = Result<V, RequestError<ConnectionError>>;

pub trait HttpRequest<C>
where C: Connection {
    fn ok<Data: Serialize>(self, data: &Data) -> RequestResult<(), C::Error>;

    fn bad_request<Data: Serialize>(self, data: &Data) -> RequestResult<(), C::Error>;

    fn not_found<Data: Serialize>(self, data: &Data) -> RequestResult<(), C::Error>;

    fn internal_server_error<Data: Serialize>(self, data: &Data) -> RequestResult<(), C::Error>;

    fn parameters<Parameters: DeserializeOwned>(&self) -> RequestResult<Parameters, C::Error>;

    fn read_all<'a, Data: DeserializeOwned>(&mut self) -> RequestResult<Data, C::Error>;
}

impl<C> HttpRequest<C> for Request<C>
where C: Connection {
    fn ok<Data: Serialize>(self, data: &Data) -> RequestResult<(), C::Error> {
        status_response(self, 200, data, "Ok", &[])
    }

    fn bad_request<Data: Serialize>(self, data: &Data) -> RequestResult<(), C::Error> {
        status_response(self, 400, data, "Bad Request", &[])
    }

    fn not_found<Data: Serialize>(self, data: &Data) -> RequestResult<(), C::Error> {
        status_response(self, 404, data, "Not found", &[])
    }

    fn internal_server_error<Data: Serialize>(self, data: &Data) -> RequestResult<(), C::Error> {
        status_response(self, 500, data, "Internal Server Error", &[])
    }

    fn parameters<Parameters: DeserializeOwned>(&self) -> RequestResult<Parameters, C::Error> {
        if let Some((_, params)) = self.uri().split_once("?") {
            serde_urlencoded::from_str::<Parameters>(params)
                .map_err(RequestError::SerdeURLError)
        } else {
            let message: String = format!("URL parameters missing. URL: {}", self.uri());
            Err(RequestError::SerdeURLError(serde_urlencoded::de::Error::custom(message)))
        }
    }

    fn read_all<'a, Data: DeserializeOwned>(&mut self) -> RequestResult<Data, C::Error> {
        let content_length: usize = self
            .header("Content-Length")
            .and_then(|content_length| content_length.parse().ok())
            .unwrap_or(0);

        let mut buffer: Vec<u8> = vec![0; content_length];
        self.read(buffer.as_mut_slice()).map_err(RequestError::ConnectionError)?;

        let data: String = String::from_utf8_lossy(&buffer).to_string();

        let data: Data = serde_json::from_str(&data).map_err(RequestError::SerdeJsonError)?;

        Ok(data)
    }
}
