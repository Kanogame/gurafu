use iced::{
    Point, Size, Theme,
    widget::{Canvas, canvas},
};

use crate::gurafuApp::canvas::camera::Camera;

pub struct Grid {
    size: Size,
    stepSize: f32,
    // elements??
}

impl Grid {
    pub fn new(size: Size) -> Self {
        Grid {
            size: size,
            stepSize: 10_f32,
        }
    }

    // generates points visible to Camera, every stepSize, using theme
    pub fn getGridPoints(&self, camera: Camera) -> Vec<canvas::Path> {
        // top left corner of camera
        let cameraTL = camera.pos;
        let size = camera.size;

        let mut points: Vec<canvas::Path> = vec![];

        // clamp TL to size
        let mut clampedCameraTL = Point {
            x: cameraTL.x % self.stepSize,
            y: cameraTL.y % self.stepSize,
        };

        while clampedCameraTL.x < size.width {
            while clampedCameraTL.y < size.height {
                points.push(canvas::Path::circle(clampedCameraTL, 5_f32));

                clampedCameraTL.y += self.stepSize;
            }

            clampedCameraTL.x += self.stepSize;
        }

        points
    }
}
