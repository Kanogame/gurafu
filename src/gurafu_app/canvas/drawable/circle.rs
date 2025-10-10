use core::fmt;

use iced::{
    widget::canvas::{Path}, Color, Point, Theme
};

use crate::gurafu_app::canvas::{camera::Camera, drawable::Drawable};


#[derive(Clone)]
pub struct Circle {
    pub center: Point,
    pub radius: f32,
    pub color: Color,
}

impl Circle {
    pub fn highlight_solution(&mut self) {
        self.color = Theme::Dark.palette().success;
    }
}

impl fmt::Debug for Circle {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return Ok(());
    }
}

impl Drawable for Circle {
    fn into_path(&self, camera: &Camera) -> Path {
        Path::circle(
            camera.world_to_screen(self.center),
            self.radius / camera.scale,
        )
    }

    fn get_color(&self) -> Color {
        return self.color;
    }
}