use super::effects::effect::Effect;
use super::frame::AnimationFrame;

#[derive(Clone)]
pub struct Animation {
    pub frames: Vec<AnimationFrame>,
    pub target_fps: u8,
}

impl Animation {
    pub fn new(image: Vec<u8>, size: (u8, u8), target_fps: u8, effect: &dyn Effect) -> Self {
        let (width, height) = (32, 32);
        let frames = effect.apply(&image, &target_fps);

        Self { frames, target_fps }
    }
}