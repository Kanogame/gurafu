use iced::{Point, Rectangle, Size};

#[derive(Debug, Clone)]
pub struct Camera {
    // position of camera in grid coordinates
    pos: Point<f32>,

    // size of camera
    size: Size<f32>,

    // camera zoom
    scale: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera{
            pos: Point { x: 0_f32, y: 0_f32 },
            size: Size { width: 800_f32, height: 600_f32 },
            scale: 1_f32,
        }
    }

    pub fn setSize(&mut self, size: Size<f32>) {
        self.size = size;
    }

    
    pub fn WorldToScreen(&self, worldCoords: Point<f32>) -> Point<f32> {
        let inv: f32 = 1_f32 / self.scale;

        Point { x: (worldCoords.x - self.pos.x) * inv, y: (worldCoords.y - self.pos.x) * inv }
    }

    //fn ScreenToWorld(&self, screen: Point<f64>) -> Point<f64> {
    //    Point { x: screen.x + self.x * self.scale, y: screen.y + self.y * self.scale }
    //}
}