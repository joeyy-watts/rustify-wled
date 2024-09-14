#[derive(Clone)]
pub struct AnimationFrame {
    pub data: Vec<u8>,
 }

impl AnimationFrame {
    pub fn new(data: &Vec<u8>) -> Self {
        Self { data: data.clone()}
    }
}
