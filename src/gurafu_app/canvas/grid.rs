use iced::{Point, Size, widget::canvas};

use crate::gurafu_app::canvas::camera::Camera;

#[derive(Debug, Default, Clone)]
pub struct Grid {
    step_size: f32,
    // elements??
}

impl Grid {
    pub fn new() -> Self {
        Grid { step_size: 100_f32 }
    }

    // generates points visible to Camera, every stepSize
    pub fn get_grid_points(&self, camera: &Camera, size: Size) -> Vec<canvas::Path> {
        // top left corner of screen in world cords
        let camera_tl = camera.scree_to_world(Point { x: 0_f32, y: 0_f32 });
        // bottom right corner of screen in world cords
        let camera_br = camera.scree_to_world(Point {
            x: size.width,
            y: size.height,
        });

        let mut points: Vec<canvas::Path> = vec![];

        let start_on_grid = self.to_grid_tl(camera_tl);
        let end_on_grid = self.to_grid_tl(camera_br);

        let mut x_offset = 0_f32;
        let mut y_offset = 0_f32;

        while start_on_grid.x + x_offset <= end_on_grid.x {
            while start_on_grid.y + y_offset <= end_on_grid.y {
                points.push(canvas::Path::circle(
                    camera.world_to_screen(Point {
                        x: start_on_grid.x + x_offset,
                        y: start_on_grid.y + y_offset,
                    }),
                    5_f32,
                ));

                y_offset += self.step_size;
            }

            y_offset = 0_f32;
            x_offset += self.step_size;
        }

        points
    }

    // clamps the point to top left point on grid
    fn to_grid_tl(&self, world: Point) -> Point {
        Point {
            x: (world.x / self.step_size).floor() * self.step_size,
            y: (world.y / self.step_size).floor() * self.step_size,
        }
    }
}
