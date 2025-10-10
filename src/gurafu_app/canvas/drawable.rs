
use iced::{
    widget::canvas::{Path}, Color
};

pub mod arrow;
pub mod circle;

use crate::gurafu_app::canvas::camera::Camera;

pub trait Drawable {
    fn into_path(&self, camera: &Camera) -> Path;
    fn get_color(&self) -> Color;
}


