use nalgebra_glm::{Vec2};

pub struct Player {
    pub pos: Vec2,
    pub a: f32,
    pub fov: f32, // fiel of view
    pub previous_mouse_pos: Vec2,
}

impl Player {
    pub fn new(x: f32, y: f32, angle: f32, fov: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            a: angle,
            fov,
            previous_mouse_pos: Vec2::new(0.0, 0.0),
        }
    }
}