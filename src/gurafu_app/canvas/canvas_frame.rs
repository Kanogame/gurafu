use iced::{widget::canvas::{self, Fill, Geometry, Path}, Renderer, Size};

use crate::gurafu_app::canvas::{camera::Camera, drawable::{DrawablePath, DrawableText}};


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

    pub fn fill_frame(&mut self, elements: Vec<&dyn DrawablePath>) {
        
        for el in elements {
            self.raw.fill(&el.into_path(&self.camera), el.get_path_color());
        }
    } 

    pub fn fill_text(&mut self, elements: Vec<&dyn DrawableText>) {
        
        for el in elements {
            self.raw.fill_text(el.into_text(&self.camera));
        }
    } 

    pub fn into_geometry(self) -> Geometry {
        self.raw.into_geometry()
    }
}

