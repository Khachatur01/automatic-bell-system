use std::sync::{Arc, LockResult, Mutex};
use esp_idf_svc::http::Method;
use disk::disk::Disk;
use http_server::http_response::HttpResponse;
use http_server::http_server::HttpServer;
use interface::Path;

const UI_FILES_LOCATION: &str = "/www";

pub fn serve(http_server: &mut HttpServer, disk: Arc<Mutex<Disk<'static>>>) {
    http_server.add_handler("/*?", Method::Get, move |request| {
        let filepath: &str = match request.uri.as_str() {
            "/" => "/index.htm",
            rest => rest
        };

        if let Ok(mut disk) = disk.lock() {
            if let Ok(path) = Path::try_from(format!("{UI_FILES_LOCATION}{filepath}")) {
                if let Ok(content) = disk.read_from_file(&path) {
                    HttpResponse::ok(content)
                } else {
                    HttpResponse::not_found(format!("File |{filepath}| doesn't found!").into())
                }
            } else {
                HttpResponse::bad_request(format!("Can't build file path |{UI_FILES_LOCATION}{filepath}|!").into())
            }
        } else {
            HttpResponse::internal_server_error("Can't lock disk mutex!".to_string().into())
        }
    }).unwrap();
}
