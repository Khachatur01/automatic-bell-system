use esp_idf_svc::http::Method;
use http_server::http_response::HttpResponse;
use http_server::http_server::HttpServer;

const API_V1: &str = "/api/v1";

pub fn serve(http_server: &mut HttpServer) {
    clock(http_server);
    alarm(http_server);
}

fn auth(http_server: &mut HttpServer) {
    http_server.add_handler(format!("{API_V1}/login").as_str(), Method::Get, move |request| {
        HttpResponse::internal_server_error("Can't lock disk mutex!".to_string().into())
    }).unwrap();

    http_server.add_handler(format!("{API_V1}/account/password").as_str(), Method::Post, move |request| {
        HttpResponse::internal_server_error("Can't lock disk mutex!".to_string().into())
    }).unwrap();
}

fn clock(http_server: &mut HttpServer) {
    http_server.add_handler(format!("{API_V1}/clock").as_str(), Method::Get, move |request| {
        HttpResponse::internal_server_error("Can't lock disk mutex!".to_string().into())
    }).unwrap();

    http_server.add_handler(format!("{API_V1}/clock").as_str(), Method::Post, move |request| {
        HttpResponse::internal_server_error("Can't lock disk mutex!".to_string().into())
    }).unwrap();
}

fn alarm(http_server: &mut HttpServer) {
    http_server.add_handler(format!("{API_V1}/alarm").as_str(), Method::Get, move |request| {
        HttpResponse::internal_server_error("Can't lock disk mutex!".to_string().into())
    }).unwrap();

    http_server.add_handler(format!("{API_V1}/alarm").as_str(), Method::Post, move |request| {
        HttpResponse::internal_server_error("Can't lock disk mutex!".to_string().into())
    }).unwrap();

    http_server.add_handler(format!("{API_V1}/alarm").as_str(), Method::Put, move |request| {
        HttpResponse::internal_server_error("Can't lock disk mutex!".to_string().into())
    }).unwrap();

    http_server.add_handler(format!("{API_V1}/alarm").as_str(), Method::Delete, move |request| {
        HttpResponse::internal_server_error("Can't lock disk mutex!".to_string().into())
    }).unwrap();
}
