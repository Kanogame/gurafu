use iced::{widget::canvas::{self, Fill, Geometry, Path}, Renderer, Size};

use crate::gurafu_app::canvas::{camera::Camera, drawable::Drawable};


pub struct CanvasFrame{
    raw: canvas::Frame,
    camera: Camera,
}

impl CanvasFrame {
    pub fn new(renderer: &Renderer, size: Size, camera: Camera) -> Self {
        Self {
            raw: canvas::Frame::new(renderer, size),
            camera: camera
        }
    }

    pub fn fill(&mut self, path: &Path, fill: impl Into<Fill>) {
            self.raw.fill(path, fill);
    } 

    pub fn fill_frame(&mut self, elements: Vec<&dyn Drawable>) {
        
        for el in elements {
            self.raw.fill(&el.into_path(&self.camera), el.get_color());
        }
    } 

    pub fn into_geometry(self) -> Geometry {
        self.raw.into_geometry()
    }
}

