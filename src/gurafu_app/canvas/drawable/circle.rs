use core::fmt;

use iced::{Color, Point, Theme, widget::canvas::Path};
use serde::{Deserialize, Serialize};

use crate::gurafu_app::{
    Node,
    canvas::{camera::Camera, drawable::Drawable},
};

impl Node {
    pub fn into_cricle(&self) -> Circle {
        Circle {
            center: Point {
                x: self.x,
                y: self.y,
            },
            radius: 30.0,
            color: Theme::Dark.palette().primary,
        }
    }
}

impl Into<Point> for &Node {
    fn into(self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}

impl From<Point> for Node {
    fn from(value: Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Clone)]
pub struct Circle {
    pub center: Point,
    pub radius: f32,
    pub color: Color,
}

impl Circle {
    pub fn highlight_start(&mut self) {
        self.color = Color::from_rgb8(0, 255, 136);
    }

    pub fn highlight_exploring(&mut self) {
        self.color = Color::from_rgb8(255, 215, 0)
    }

    pub fn highlight_candidate(&mut self) {
        self.color = Color::from_rgb8(135, 206, 235)
    }

    pub fn highlight_next(&mut self) {
        self.color = Color::from_rgb8(255, 140, 0)
    }

    pub fn highlight_visited(&mut self) {
        self.color = Color::from_rgb8(136, 136, 136)
    }

    pub fn highlight_solution(&mut self) {
        self.color = Color::from_rgb8(0, 255, 0)
    }

    pub fn reset_highlight(&mut self) {
        self.color = Theme::Dark.palette().primary;
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
