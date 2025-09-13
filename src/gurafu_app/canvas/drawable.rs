use iced::{
    Point,
    widget::canvas::{Path},
};

use crate::gurafu_app::canvas::camera::Camera;

pub trait Drawable {
    fn into_path(&self, world_position: Point, camera: &Camera) -> Path;
}

pub struct Circle {
    pub radius: f32,
}

impl Drawable for Circle {
    fn into_path(&self, world_position: Point, camera: &Camera) -> Path {
        Path::circle(camera.world_to_screen(world_position), self.radius / camera.scale)
    }
}
