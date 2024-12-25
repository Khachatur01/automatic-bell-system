use crate::schedule_system::ScheduleSystem;
use esp_idf_svc::http::Method;
use http_server::http_request::HttpRequest;
use interface::Path;
use std::sync::{Arc, Mutex, RwLock};
use esp_idf_svc::sys::EspError;
use http_server::http_server::HttpServer;

const UI_FILES_LOCATION: &str = "/www";

pub fn serve(http_server: &mut HttpServer, schedule_system: Arc<ScheduleSystem>) -> Result<(), EspError> {
    let mut schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);

    http_server.add_handler("/*?", Method::Get, move |request| {
        let filepath: String = match request.uri() {
            "/" => String::from("/index.htm"),
            rest => String::from(rest),
        };

        if let Ok(path) = Path::try_from(format!("{UI_FILES_LOCATION}{filepath}")) {
            match schedule_system_clone.read_from_file(&path) {
                Ok(content) => request.ok(&String::from_utf8_lossy(&content)),
                Err(error) => request.not_found(&format!("Can't read from file |{UI_FILES_LOCATION}{filepath}|. {error}"))
            }
        } else {
            request.bad_request(&format!("Can't build file path |{UI_FILES_LOCATION}{filepath}|!"))
        }
    })
}
