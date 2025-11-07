use iced::{Color, widget::canvas::{Path, Text}};

pub mod arrow;
pub mod circle;

use crate::gurafu_app::canvas::camera::Camera;

pub trait DrawablePath {
    fn into_path(&self, camera: &Camera) -> Path;
    fn get_path_color(&self) -> Color;
}

pub trait DrawableText {
    fn into_text(&self, camera: &Camera) -> Text;
}
