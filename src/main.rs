#[macro_use] extern crate cached;

use actix_files::NamedFile;
use validator::Validate;
use actix_web::{web, middleware, guard, http, App, HttpResponse, HttpServer, Result};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde_derive::{Deserialize, Serialize};

#[derive(Validate, Serialize, Deserialize)]
struct Request {
    #[validate(length(min = "1", max = "1000000"))]
    domain: String
}

#[derive(Serialize, Deserialize)]
struct Response {
    supported: bool
}

fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
}

fn p404() -> Result<NamedFile> {
    Ok(NamedFile::open("static/404.html")?.set_status_code(http::StatusCode::NOT_FOUND))
}

fn support(
    request: web::Json<Request>
) -> HttpResponse {
    let domain = &request.domain;
    HttpResponse::Ok().json(Response {
        supported: check(domain.to_string())
    })
}

cached!{
    CHECK;
    fn check(domain: String) -> bool = {
        true
    }
}

fn main() {
    // load ssl keys
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    let endpoint = "127.0.0.1:8080";

    println!("Starting server at: {:?}", endpoint);

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::NormalizePath)
            .wrap(middleware::Compress::default())
            .route("/", web::get().to_async(index))
            .route("/support", web::post().to_async(support))
            .default_service(
                // 404 for GET request
                web::resource("")
                    .route(web::get().to(p404))
                    // All requests that are not `GET`
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind_ssl(endpoint, builder)
    .unwrap()
    .run()
    .unwrap();
}