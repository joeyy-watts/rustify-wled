#[macro_use] extern crate rocket;

use std::sync::mpsc;

use rocket::response::Redirect;
use rocket::futures::future::Either;
use rocket::State;
use rustify_wled_lib::lib::controllers::animation::{AnimationController, AnimationControllerConfig};
use rustify_wled_lib::lib::controllers::app::ApplicationController;
use rustify_wled_lib::lib::controllers::spotify::SpotifyController;
use rustify_wled_lib::lib::models::app_channels::AppChannels;
use rustify_wled_lib::utils::network::resolve_ip;

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
    let channels: AppChannels = AppChannels::setup();

    let animation_controller: AnimationController = AnimationController::new(channels.anim_msg_rx);
    let spotify_controller: SpotifyController = SpotifyController::new(channels.playback_tx, channels.sp_msg_rx);
    let app_controller: ApplicationController = ApplicationController::new(
        animation_controller,
        spotify_controller,
        channels.playback_rx,
        channels.sp_msg_tx,
        channels.anim_msg_tx,
    );


    rocket::build()
    .mount("/", routes![anim_start])
    .mount("/", routes![anim_stop])
    .mount("/", routes![callback])
    .manage(app_controller)
}