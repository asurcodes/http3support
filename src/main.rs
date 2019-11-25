#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

#[cfg(test)] mod tests;

use std::process::Command;

use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::serve::StaticFiles;

#[derive(Serialize, Deserialize)]
struct FormInput {
    url: String
}

#[post("/", format = "json", data = "<form>")]
fn check(form: Json<FormInput>) -> JsonValue {
    let url = &form.url;
    json!({
        "supported": support(url.to_string()),
        "url": url.to_string()
    })
}

#[catch(404)]
fn not_found() -> String {
    format!("Sorry, page not found!")
}

fn support (url: String) -> bool {
    let status = Command::new("./http3client")
        .arg(url)
        .status()
        .expect("failed to execute request");

    status.success()
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