use super::effects::base::brightness::BrightnessEffect;
use super::frame::AnimationFrame;
use super::effects::base::r#static::StaticEffect;

#[derive(Clone)]
pub struct Animation {
    pub frames: Vec<AnimationFrame>,
}

impl Animation {
    pub fn new(image: Vec<u8>, size: (u8, u8), effect: BrightnessEffect) -> Self {
        let (width, height) = (32, 32);
        let frames = effect.apply(&image);

        Self { frames }
    }
}