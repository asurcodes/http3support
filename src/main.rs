#[macro_use] extern crate validator_derive;
#[macro_use] extern crate cached;

use validator::Validate;
use serde_derive::{Deserialize, Serialize};
use futures::{Future};
use actix_files::NamedFile;
use actix_web::{web, middleware, guard, http, error::ErrorBadRequest, App, Error, HttpResponse, HttpServer, Result};
use rustls::internal::pemfile::{certs, rsa_private_keys};
use rustls::{NoClientAuth, ServerConfig};

#[derive(Validate, Serialize, Deserialize)]
struct Request {
    #[validate(url, length(min = "10", max = "100"))]
    url: String
}

#[derive(Serialize, Deserialize)]
struct Response {
    supported: bool,
    url: String
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

    // let url = Url::parse(request.domain.to_string())?;
    // let base = base_url(url)?;

    let response = Response {
        supported: check(request.url.to_string()),
        url: request.url.to_string()
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

fn load_ssl() -> ServerConfig {
    use std::io::BufReader;

    const CERT: &'static [u8] = include_bytes!("../cert.pem");
    const KEY: &'static [u8] = include_bytes!("../key.pem");

    let mut cert = BufReader::new(CERT);
    let mut key = BufReader::new(KEY);

    let mut config = ServerConfig::new(NoClientAuth::new());
    let cert_chain = certs(&mut cert).unwrap();
    let mut keys = rsa_private_keys(&mut key).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

    config
}

fn main() -> std::io::Result<()> {
    // For testing: `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let endpoint = "127.0.0.1:4433";
    let config = load_ssl();
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
    .bind_rustls(endpoint, config)?
    .run()
}