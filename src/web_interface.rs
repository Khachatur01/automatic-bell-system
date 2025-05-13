use crate::schedule_system::ScheduleSystem;
use esp_idf_svc::http::Method;
use esp_idf_svc::sys::EspError;
use http_server::http_request::{IntoResponse, RequestError};
use http_server::http_server::HttpServer;
use std::sync::Arc;
use esp_idf_svc::http::server::Response;
use esp_idf_svc::io::{EspIOError, Write};
use mime::Mime;
use mime_guess::MimeGuess;
use crate::constant::{SYSTEM_DIR, WEB_UI_DIR};

pub fn serve(http_server: &mut HttpServer, schedule_system: Arc<ScheduleSystem>) -> Result<(), EspError> {
    let ui_files_location: String = format!("/{SYSTEM_DIR}/{WEB_UI_DIR}");

    let schedule_system_clone: Arc<ScheduleSystem> = Arc::clone(&schedule_system);

    http_server.add_handler("/*?", Method::Get, move |request| {
        let filepath: &str =
            match request.uri().rsplit_once(".") {
                None => "/",
                Some((_, extension)) => {
                    if extension.contains("/") {
                        "/"
                    } else {
                        request.uri()
                    }
                }
            };

        let filepath: String = match filepath {
            "/" => String::from("/index.htm"),
            rest => String::from(rest),
        };

        let Ok(path) = format!("{ui_files_location}{filepath}").as_str().try_into() else {
            return request.bad_request(&format!("Can't build file path |{ui_files_location}{filepath}|!"))
        };
        println!("{:?}", path);

        let Some(guess) = mime_guess::from_path(filepath).first() else {
            return request.bad_request(&format!("Can't build file |{ui_files_location}"))
        };

        let mime_type: String = format!("{}; charset=utf-8", guess);
        println!("{:?}", mime_type);

        let headers = &[
            ("Content-Type", mime_type.as_str()),

            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH"),
            ("Access-Control-Allow-Headers", "Content-Type"),
        ];

        let mut response = request
            .into_response(200, Some("Sending chunks"), headers)
            .map_err(RequestError::Connection)?;


        let _ = schedule_system_clone.read_from_file_bytes(
            &path,
            64 * 1024,
            |buffer, bytes_read| {
                match response.write(&buffer[0..bytes_read]) {
                    Ok(written_bytes) => {
                        println!("{written_bytes} bytes written");
                        Ok(())
                    }
                    Err(error) => {
                        eprintln!("{error}");
                        Err(())
                    }
                }
            });

        response.flush().map_err(RequestError::Connection)
    })
}
