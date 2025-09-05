use iced::Point;

pub struct Grid {
    x: f64,
    y: f64,
    scale: f64
}

impl Grid {
    fn new() -> Self {
        return Grid{
            x: 0_f64,
            y: 0_f64,
            scale: 1_f32,
        };
    }

    fn worldToScreen()

    fn screenToWorld(&self, screen: Point<f64>) -> Point<f64> {
        let inv: f64 = 1_f64 / self.scale;

        Point { x: (screen.x - self.x) * inv, y: (screen.y - self.y) * inv }

    }
}