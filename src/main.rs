use actix_web::{web, middleware, App, HttpResponse, HttpServer, Responder, Result};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix_files::NamedFile;

fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
}

fn support() -> impl Responder {
    HttpResponse::Ok().body("Supports HTTP/3!")
}

fn main() {
    // load ssl keys
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::NormalizePath)
            .wrap(middleware::Compress::default())
            .route("/", web::get().to(index))
            .route("/support", web::get().to(support))
    })
    .bind_ssl("127.0.0.1:8088", builder)
    .unwrap()
    .run()
    .unwrap();
}