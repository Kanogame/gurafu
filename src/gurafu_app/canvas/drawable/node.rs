use core::fmt;

use crate::gurafu_app::{canvas::{
    camera::{Camera, WorldPoint},
    drawable::{DrawablePath, DrawableText},
}, styles::get_theme};
use iced::{
    Color, Font, Pixels, alignment,
    widget::canvas::{Path, Text},
};
use serde::{Deserialize, Serialize};

impl Into<Node> for &NodeSerializable {
    fn into(self) -> Node {
        Node {
            id: 0,
            center: self.position,
            radius: 30.0,
            color: get_theme().palette().primary,
        }
    }
}

impl Into<WorldPoint> for &NodeSerializable {
    fn into(self) -> WorldPoint {
        self.position
    }
}

impl From<WorldPoint> for NodeSerializable {
    fn from(value: WorldPoint) -> Self {
        Self { position: value }
    }
}

#[derive(Clone, Default)]
pub struct Node {
    pub id: usize,
    pub center: WorldPoint,
    pub radius: f32,
    pub color: Color,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeSerializable {
    position: WorldPoint,
}

impl Node {
    pub fn highlight_start(&mut self) {
        self.color = Color::from_rgb8(0, 255, 136);
    }

     pub fn highlight_current(&mut self) {
        self.color = Color::from_rgb8(255, 255, 136);
    }

    pub fn highlight_error(&mut self) {
        self.color = Color::from_rgb8(255, 0, 0);
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
        self.color = get_theme().palette().primary;
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return Ok(());
    }
}

impl DrawablePath for Node {
    fn into_path(&self, camera: &Camera) -> Path {
        Path::circle(
            camera.world_to_screen(self.center),
            self.radius / camera.scale,
        )
    }

    fn get_path_color(&self) -> Color {
        return self.color;
    }
}

impl DrawableText for Node {
    fn into_text(&self, camera: &Camera) -> Text {
        let font_size = 20.0;

        Text {
            content: self.id.to_string(),
            position: camera.world_to_screen(self.center),
            color: Color::BLACK,
            size: Pixels::from(font_size / camera.scale),
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            font: Font {
                weight: iced::font::Weight::Bold,
                ..Font::default()
            },
            ..Text::default()
        }
    }
}
