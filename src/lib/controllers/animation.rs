use crate::lib::controllers::artnet::ArtNetController2D;
use crate::lib::artnet::anim::animation::Animation;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;


/// Controller for playing animations to target ArtNet devices
/// 
/// `artnet_controller` - the controller for the target ArtNet device
/// `active_animation` - thread of the currently playing animation
/// 
pub struct AnimationController {
    pub active_animation: Arc<Mutex<Option<Animation>>>,
    artnet_controller: Arc<ArtNetController2D>,
    stop_flag: Arc::<AtomicBool>
}

impl AnimationController {
    pub fn new(target: String, dimensions: (u16, u16)) -> Self {
        let artnet_controller = ArtNetController2D::new(target, dimensions);
        Self { 
            active_animation: Arc::new(Mutex::new(None)), 
            artnet_controller: Arc::new(artnet_controller),
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Plays the given animation to the target device
    /// 
    /// `animation` - the animation to be played
    /// 
    /// Returns:
    ///     A Result indicating the success of the operation
    /// 
    pub fn play_animation(&mut self, animation: Animation) {
        // if some animation is already playing, stop it
        if !self.active_animation.lock().unwrap().is_none() {
            self.stop_animation();
        }

        self.active_animation.lock().unwrap().replace(animation);

        // clone needed fields for thread
        let cloned_controller = Arc::clone(&self.artnet_controller);
        let cloned_animation = Arc::clone(&self.active_animation);
        // this probably shouldn't be cloned
        let cloned_stop_flag = Arc::clone(&self.stop_flag);

        thread::spawn(move || loop {
                for frame in cloned_animation.lock().unwrap().as_mut().unwrap().frames.clone() {
                    cloned_controller.send_frame(frame);
                    // TODO: add logic for appropriate delay
                    thread::sleep(Duration::from_millis(200));
                }

                if cloned_stop_flag.load(Ordering::Relaxed) == true {
                    cloned_stop_flag.store(false, Ordering::Relaxed);
                    break;
                }
            }  
        );
    }

    /// Stops the currently playing animation
    /// 
    /// Returns:
    ///     A Result indicating the success of the operation
    ///
    pub fn stop_animation(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}