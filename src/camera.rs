use glam::*;

pub struct Camera {
    pub pos: Vec2,
    pub size: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            pos: Vec2::new(0.0, 0.0),
            size: 1.0,
        }
    }
    pub fn matrix(&self, width: u32, height: u32) -> Mat4 {
        Mat4::orthographic_rh_gl(
            self.pos.x,
            self.pos.x + width as f32,
            self.pos.y + height as f32,
            self.pos.y,
            -1.0,
            1.0,
        )
    }
}
