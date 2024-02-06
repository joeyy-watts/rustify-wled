#[macro_use] extern crate rocket;

use rustify_wled_lib::lib::artnet::anim::animation::Animation;
use rustify_wled_lib::lib::artnet::anim::effects::base::brightness::BrightnessEffect;
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

fn start_loop() {
    let mut animation_controller: AnimationController = AnimationController::new(String::from("192.168.31.87"), (32, 32));

    let animation: Animation = Animation::new(get_test_array(), (32, 32), 12, &StaticEffect);

    animation_controller.play_animation(animation);
}

#[get("/start")]
fn artnet_start() -> &'static str {
    start_loop();
    "start"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
    .mount("/", routes![index])
    .mount("/", routes![artnet_start])
}