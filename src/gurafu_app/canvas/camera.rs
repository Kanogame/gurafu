use iced::Point;

#[derive(Debug, Clone)]
pub struct Camera {
    // position of camera in grid coordinates
    pub pos: Point<f32>,

    // camera zoom
    scale: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            pos: Point { x: 0_f32, y: 0_f32 },
            scale: 1_f32,
        }
    }
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            pos: Point { x: 1_f32, y: 0_f32 },
            scale: 1_f32,
        }
    }

    pub fn world_to_screen(&self, world_coords: Point<f32>) -> Point<f32> {
        let inv: f32 = 1_f32 / self.scale;

        Point {
            x: (world_coords.x - self.pos.x) * inv,
            y: (world_coords.y - self.pos.y) * inv,
        }
    }

    pub fn apply_drag(&mut self, offset: Point) {
        self.pos.x += offset.x;
        self.pos.y += offset.y;
    }

    pub fn apply_scroll(&mut self, scroll: f32) {
        self.scale += scroll;
    }

    pub fn scree_to_world(&self, screen_coords: Point) -> Point {
        Point {
            x: screen_coords.x + self.pos.x * self.scale,
            y: screen_coords.y + self.pos.y * self.scale,
        }
    }
}
