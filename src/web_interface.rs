use crate::schedule_system::ScheduleSystem;
use esp_idf_svc::http::Method;
use esp_idf_svc::sys::EspError;
use http_server::http_request::{IntoResponse};
use http_server::http_server::HttpServer;
use std::sync::Arc;
use serde_json::value::RawValue;
use http_server::http_request::ResponseData;

const UI_FILES_LOCATION: &str = "/schedule/www";

pub fn serve(http_server: &mut HttpServer, schedule_system: Arc<ScheduleSystem>) -> Result<(), EspError> {
    let mut schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);

    http_server.add_handler("/*?", Method::Get, move |request| {
        let filepath: String = match request.uri() {
            "/" => String::from("/index.htm"),
            rest => String::from(rest),
        };

        let Ok(path) = format!("{UI_FILES_LOCATION}{filepath}").as_str().try_into() else {
            return request.bad_request(&format!("Can't build file path |{UI_FILES_LOCATION}{filepath}|!").as_str())
        };

        match schedule_system_clone.read_from_file(&path) {
            Ok(content) => request.ok(&RawValue::from_string(String::from_utf8_lossy(&content).to_string()).unwrap().to_string()),
            Err(error) => request.not_found(&format!("Can't read from file |{UI_FILES_LOCATION}{filepath}|. {error}").as_str())
        }
    })
}
