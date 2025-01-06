use esp_idf_svc::http::server::{Connection, Request, Response};
use esp_idf_svc::io::Write;
use serde::de::{DeserializeOwned, Error as _};
use serde::{Serialize};
use crate::to_response_data::ToResponseData;

fn status_response<'a, C, Data>(mut request: Request<C>,
                                status: u16,
                                data: &Data,
                                message: &'a str,
                                headers: &'a [(&'a str, &'a str)],) -> RequestResult<(), C::Error>
where C: Connection,
      Data: ToResponseData {

    let mut response: Response<C> = request
        .into_response(status, Some(message), headers)
        .map_err(RequestError::Connection)?;

    let response_str = data.to_response_data();

    let response_data: &[u8] = response_str.as_bytes();

    response.write_all(response_data)
        .map_err(RequestError::Connection)?;

    Ok(())
}

#[derive(Debug)]
pub enum RequestError<ConnectionError> {
    SerdeJson(serde_json::Error),
    SerdeURL(serde_urlencoded::de::Error),
    Connection(ConnectionError),
    General(String),
}

pub type RequestResult<V, ConnectionError> = Result<V, RequestError<ConnectionError>>;

pub enum ResponseData<Data: Serialize + ToString> {
    Json(Data),
    Str(Data),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
pub trait IntoResponse<C>
where C: Connection,
      Self: Sized {
    fn ok<Data: ToResponseData>(self, data: &Data) -> RequestResult<(), C::Error>;

    fn bad_request<Data: ToResponseData>(self, data: &Data) -> RequestResult<(), C::Error>;

    fn not_found<Data: ToResponseData>(self, data: &Data) -> RequestResult<(), C::Error>;

    fn internal_server_error<Data: ToResponseData>(self, data: &Data) -> RequestResult<(), C::Error>;
}

impl<C> IntoResponse<C> for Request<C>
where C: Connection {
    fn ok<Data: ToResponseData>(self, data: &Data) -> RequestResult<(), C::Error> {
        status_response(self, 200, data, "Ok", &[])
    }

    fn bad_request<Data: ToResponseData>(self, data: &Data) -> RequestResult<(), C::Error> {
        status_response(self, 400, data, "Bad Request", &[])
    }

    fn not_found<Data: ToResponseData>(self, data: &Data) -> RequestResult<(), C::Error> {
        status_response(self, 404, data, "Not found", &[])
    }

    fn internal_server_error<Data: ToResponseData>(self, data: &Data) -> RequestResult<(), C::Error> {
        status_response(self, 500, data, "Internal Server Error", &[])
    }
}


////////////////////////////////////////////////////////////////////////////////////////////////////
pub trait ReadParameters<C>
where C: Connection {
    fn parameters<Parameters: DeserializeOwned>(&self) -> RequestResult<Parameters, C::Error>;
}

impl<C> ReadParameters<C> for Request<C>
where C: Connection {
    /**
    * URI example: /api/v1/resource?param1=value1&param2=value2
    * Method serde_urlencoded::from_str() gets string containing everything after '?' symbol.
    */
    fn parameters<Parameters: DeserializeOwned>(&self) -> RequestResult<Parameters, C::Error> {
        if let Some((_, params)) = self.uri().split_once("?") {
            serde_urlencoded::from_str::<Parameters>(params)
                .map_err(RequestError::SerdeURL)
        } else {
            let message: String = format!("URL parameters missing. URL: {}", self.uri());
            Err(RequestError::SerdeURL(serde_urlencoded::de::Error::custom(message)))
        }
    }
}


////////////////////////////////////////////////////////////////////////////////////////////////////
pub trait ReadData<C>
where C: Connection {
    fn read_all<'a, Data: DeserializeOwned>(&mut self) -> RequestResult<Data, C::Error>;
}

impl<C> ReadData<C> for Request<C>
where C: Connection {
    fn read_all<'a, Data: DeserializeOwned>(&mut self) -> RequestResult<Data, C::Error> {
        let content_length: usize = self
            .header("Content-Length")
            .and_then(|content_length| content_length.parse().ok())
            .unwrap_or(0);

        let mut buffer: Vec<u8> = vec![0; content_length];
        self.read(buffer.as_mut_slice()).map_err(RequestError::Connection)?;

        let data: String = String::from_utf8_lossy(&buffer).to_string();

        let data: Data = serde_json::from_str(&data).map_err(RequestError::SerdeJson)?;

        Ok(data)
    }
}
