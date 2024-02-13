#[macro_use] extern crate rocket;

use rocket::response::Redirect;
use rocket::futures::future::Either;
use rocket::State;
use rustify_wled_lib::lib::controllers::app::ApplicationController;


///
/// Responses for starting the application,
/// Can be Redirect or OK
#[derive(Debug, Responder)]
pub enum StartResponses {
    Redirect(Redirect),
    String(String),
}


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/start")]
fn anim_start(controller: &State<ApplicationController>) -> StartResponses {
    match controller.start().unwrap() {
        Either::Left(redirect) => StartResponses::Redirect(redirect),
        Either::Right(string) => StartResponses::String(string)
    }
}

#[get("/stop")]
fn anim_stop(controller: &State<ApplicationController>) -> &'static str {
    controller.stop();
    "stop"
}

#[get("/callback?<code>")]
fn callback(controller: &State<ApplicationController>, code: String) -> StartResponses {
    match controller.callback(code.as_str()) {
        Ok(_) => StartResponses::Redirect(Redirect::to(uri!("/start"))),
        Err(_) => StartResponses::String(String::from("callback failed!"))
    }
}

#[launch]
fn rocket() -> _ {
    let app_controller: ApplicationController = ApplicationController::new(String::from("192.168.31.87"), (32, 32));

    rocket::build()
    .mount("/", routes![anim_start])
    .mount("/", routes![anim_stop])
    .mount("/", routes![callback])
    .manage(app_controller)
}