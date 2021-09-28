use image::{ImageBuffer,
            Rgb};

#[derive(Clone)]
pub struct ImageSize {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub struct Parameters {
    pub size:       ImageSize,
    pub position:   Position,
    pub scale:      f32,
    pub iterations: u32,
}
/// will generate a julia set with given parameters
pub fn spawn(_cores: u32, _parameters: Parameters) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    unimplemented!()
}
