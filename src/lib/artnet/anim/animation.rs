use super::effects::effect::RenderedEffect;
use super::frame::AnimationFrame;

#[derive(Clone)]
pub struct Animation {
    pub frames: Vec<AnimationFrame>,
    pub target_fps: u8,
}

impl Animation {
    pub fn new(image: Vec<u8>, target_fps: u8, effect: RenderedEffect) -> Self {
        let frames = effect.apply(&image);

        Self { frames, target_fps }
    }
}