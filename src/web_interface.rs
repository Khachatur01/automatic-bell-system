use crate::schedule_system::ScheduleSystem;
use esp_idf_svc::http::Method;
use esp_idf_svc::sys::EspError;
use http_server::http_request::{IntoResponse, RequestError};
use http_server::http_server::HttpServer;
use std::sync::Arc;
use esp_idf_svc::http::server::Response;
use esp_idf_svc::io::Write;
use crate::constant::{SYSTEM_DIR, WEB_UI_DIR};

pub fn serve(http_server: &mut HttpServer, schedule_system: Arc<ScheduleSystem>) -> Result<(), EspError> {
    let ui_files_location: String = format!("/{SYSTEM_DIR}/{WEB_UI_DIR}");

    let schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);

    http_server.add_handler("/*?", Method::Get, move |request| {
        let filepath: String = match request.uri() {
            "/" => String::from("/index.htm"),
            rest => String::from(rest),
        };

        let Ok(path) = format!("{ui_files_location}{filepath}").as_str().try_into() else {
            return request.bad_request(&format!("Can't build file path |{ui_files_location}{filepath}|!"))
        };
        println!("{:?}", path);

        // let file_read_result = schedule_system_clone.read_from_file(&path);
        // 
        // let Ok(content) = file_read_result else {
        //     let error = file_read_result.err().unwrap();
        // 
        //     println!("Can't read from file |{ui_files_location}{filepath}|. {error}");
        //     return request.not_found(&format!("Can't read from file |{ui_files_location}{filepath}|. {error}"));
        // };
        // 
        // let content: String = String::from_utf8_lossy(&content).to_string();
        // println!("{:?}", content);
        // 
        // request.ok(&content)

        let cors_headers = &[
            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH"),
            ("Access-Control-Allow-Headers", "Content-Type"),
        ];

        let mut response = request
            .into_response(200, Some("Sending chunks"), cors_headers)
            .map_err(RequestError::Connection)?;


        let _ = schedule_system_clone.read_from_file_bytes(
            &path,
            128,
            |buffer| {
                println!("buffer {:?}", buffer);
                let _ = response
                    .write(buffer)
                    .map_err(RequestError::Connection);
            });

        response.flush().map_err(RequestError::Connection)
    })
}
