use crate::schedule_system::ScheduleSystem;
use esp_idf_svc::http::Method;
use http_server::http_request::HttpRequest;
use interface::Path;
use std::sync::{Arc, Mutex};
use esp_idf_svc::sys::EspError;
use http_server::http_server::HttpServer;

const UI_FILES_LOCATION: &str = "/www";

pub fn serve(http_server: &mut HttpServer, schedule_system: Arc<Mutex<ScheduleSystem>>) -> Result<(), EspError> {
    let schedule_system_clone: Arc<Mutex<ScheduleSystem>> = Arc::clone(&schedule_system);

    http_server.add_handler("/*?", Method::Get, move |request| {
        let filepath: String = match request.uri() {
            "/" => String::from("/index.htm"),
            rest => String::from(rest),
        };

        if let Ok(mut schedule_system) = schedule_system_clone.lock() {
            if let Ok(path) = Path::try_from(format!("{UI_FILES_LOCATION}{filepath}")) {
                if let Ok(content) = schedule_system.read_from_file(&path) {
                    request.ok(content)
                } else {
                    request.not_found(format!("File |{filepath}| doesn't found!").into())
                }
            } else {
                request.bad_request(format!("Can't build file path |{UI_FILES_LOCATION}{filepath}|!").into())
            }
        } else {
            request.internal_server_error("Can't lock disk mutex!".to_string().into())
        }
    })
}
