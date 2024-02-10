#[macro_use] extern crate rocket;

use std::thread::{self, Thread};
use std::time::Duration;

use rustify_wled_lib::lib::artnet::anim::animation::Animation;
use rustify_wled_lib::lib::artnet::anim::effects::base::brightness::SinBrightnessEffect;
use rustify_wled_lib::lib::artnet::anim::effects::base::r#static::StaticEffect;
use rustify_wled_lib::lib::controllers::animation::AnimationController;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn get_test_array() -> Vec<u8> {
    // Number of LEDs
    let num_leds = 1024;

    // Initialize the array with zeros
    let mut rgb_data = vec![0; num_leds * 3];

    // Set all LEDs to red (255, 0, 0)
    for i in 0..num_leds {
        let base_index = i * 3;
        rgb_data[base_index] = 255;     // Red
        rgb_data[base_index + 1] = 0;   // Green
        rgb_data[base_index + 2] = 0;   // Blue
    }

    rgb_data
}

#[get("/start")]
fn anim_start() -> &'static str {
    let mut animation_controller: AnimationController = AnimationController::new(String::from("192.168.31.87"), (32, 32));

    let animation: Animation = Animation::new(get_test_array(), (32, 32), 24, &SinBrightnessEffect { period: 1.0, amplitude: 0.5, offset: 0.5 });

    animation_controller.play_animation(animation);

    "start"
}

#[get("/stop")]
fn anim_stop() -> &'static str {
    // animation_controller.stop_animation();
    "stop"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
    .mount("/", routes![index])
    .mount("/", routes![anim_start])
    .mount("/", routes![anim_stop])
}