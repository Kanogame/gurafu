use std::collections::HashMap;

use iced::{Point, Size, widget::canvas};
use petgraph::graph::NodeIndex;

use crate::gurafu_app::canvas::camera::Camera;

pub struct Grid {
    step_size: f32,

    points: Vec<canvas::Path>,

    objects: HashMap<GridPoint, NodeIndex>,
}

#[derive(Clone, Copy, Debug)]
pub struct GridPoint(pub iced::Point<i32>);

impl std::hash::Hash for GridPoint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.x.hash(state);
        self.0.y.hash(state);
    }
}

impl PartialEq for GridPoint {
    fn eq(&self, other: &GridPoint) -> bool {
        return self.0.x == other.0.x && self.0.y == other.0.y;
    }
}

impl Eq for GridPoint {}

impl Grid {
    pub fn new() -> Self {
        Grid {
            step_size: 100_f32,
            objects: HashMap::new(),
            points: Vec::new(),
        }
    }

    //pub fn objects(&self) -> Iter<'_, GridPoint, NodeIndex> {
    //    self.objects.iter()
    //}

    pub fn update_grid_points(&mut self, camera: &Camera, size: Size) {
        // top left corner of screen in world cords
        let camera_tl = camera.screen_to_world(Point { x: 0_f32, y: 0_f32 });
        // bottom right corner of screen in world cords
        let camera_br = camera.screen_to_world(Point {
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
                    5_f32 * (1_f32 / camera.scale),
                ));

                y_offset += self.step_size;
            }

            y_offset = 0_f32;
            x_offset += self.step_size;
        }

        self.points = points;
    }

    pub fn get_grid_points(&self) -> &Vec<canvas::Path> {
        &self.points
    }

    pub fn get_object_in_world(&self, world: Point) -> Option<&NodeIndex> {
        self.objects.get(&self.to_gridpoint(world))
    }

    // clamps the point to top left point on grid
    fn to_grid_tl(&self, world: Point) -> Point {
        Point {
            x: (world.x / self.step_size).floor() * self.step_size,
            y: (world.y / self.step_size).floor() * self.step_size,
        }
    }

    // claps the point to closest point on grid
    pub fn to_gridpoint(&self, world: Point) -> GridPoint {
        GridPoint {
            0: Point {
                x: ((world.x / self.step_size).round() * self.step_size) as i32,
                y: ((world.y / self.step_size).round() * self.step_size) as i32,
            },
        }
    }

    pub fn to_grid(&self, world: Point) -> Point {
        Point {
            x: (world.x / self.step_size).round() * self.step_size,
            y: (world.y / self.step_size).round() * self.step_size,
        }
    }

    pub fn add_to_grid(&mut self, world: Point, object: NodeIndex) {
        let grid = self.to_gridpoint(world);
        if !self.objects.contains_key(&grid) {
            self.objects.insert(grid, object);
        }
    }

    pub fn remove_from_grid(&mut self, world: Point) {
        self.objects.remove(&self.to_gridpoint(world));
    }
}
