#[macro_use] extern crate validator_derive;
#[macro_use] extern crate cached;

use actix_files::NamedFile;
use validator::Validate;
use actix_web::{web, middleware, guard, http, error::ErrorBadRequest, App, Error, HttpResponse, HttpServer, Result};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde_derive::{Deserialize, Serialize};
use futures::{Future};

#[derive(Validate, Serialize, Deserialize)]
struct Request {
    #[validate(url, length(min = "10", max = "100"))]
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
    Ok(NamedFile::open("static/404.html")?
        .set_status_code(http::StatusCode::NOT_FOUND))
}

fn support(
    request: web::Json<Request>
) -> impl Future<Item = HttpResponse, Error = Error> {
    let validation = futures::future::result(request.validate()).map_err(ErrorBadRequest);
    let response = Response {
        supported: check(request.domain.to_string())
    };
    validation.and_then(|_|
        Ok(HttpResponse::Ok().json(response))
    )
}

cached!{
    CHECK;
    fn check(domain: String) -> bool = {
        // Call quiche command to find out if domain supports http/3
        true
    }
}

fn main() {
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let endpoint = "127.0.0.1:8080";
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();
    // Start server
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::NormalizePath)
            .wrap(middleware::Compress::default())
            .service(web::resource("/").route(web::get().to_async(index)))
            .service(web::resource("/support").route(web::post().to_async(support)))
            .default_service(
                // 404 for GET request
                web::resource("")
                    .route(web::get().to(p404))
                    // all requests that are not `GET`
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