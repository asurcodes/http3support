#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

#[cfg(test)] mod tests;

use url::{Url, ParseError};
use std::process::Command;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

#[derive(Serialize, Deserialize)]
struct FormInput {
    url: String
}

#[derive(Serialize, Deserialize)]
struct Response {
    supported: bool,
    url: String
}

#[post("/", format = "json", data = "<form>")]
fn check(form: Json<FormInput>) -> Result<Json<Response>, ParseError> {
    let url = Url::parse(&form.url)?;
    let scheme = url.scheme();
    let host = url.host_str().unwrap();
    let base_url = format!("{}://{}", scheme, host);
    let s_url = base_url.trim_matches(';');

    Ok(Json(Response {
            supported: support(s_url),
            url: s_url.to_string()
        }
    ))
}

#[catch(404)]
fn not_found() -> String {
    format!("Sorry, page not found!")
}

fn support (url: &str) -> bool {
    let output = Command::new("http3-client")
        .arg(url.to_string())
        .output()
        .unwrap();

    !String::from_utf8(output.stderr).unwrap().contains("ERROR")
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", StaticFiles::from("static"))
        .mount("/check", routes![check])
        .register(catchers![not_found])
}

fn main() {
    rocket().launch();
}