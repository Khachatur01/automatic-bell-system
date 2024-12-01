pub struct HttpResponse {
    pub(crate) status: u16,
    pub(crate) message: Option<String>,
    pub(crate) data: Vec<u8>
}

impl HttpResponse {
    pub fn ok(data: Vec<u8>) -> Self {
        HttpResponse::status(200, "OK", data)
    }

    pub fn not_found(data: Vec<u8>) -> Self {
        HttpResponse::status(404, "Not Found", data)
    }

    pub fn internal_server_error(data: Vec<u8>) -> Self {
        HttpResponse::status(500, "Internal Server Error", data)
    }

    pub fn bad_request(data: Vec<u8>) -> Self {
        HttpResponse::status(400, "Bad Request", data)
    }

    fn status(status_code: u16, message: &str, data: Vec<u8>) -> Self {
        Self {
            status: status_code,
            message: Some(String::from(message)),
            data
        }
    }
}
