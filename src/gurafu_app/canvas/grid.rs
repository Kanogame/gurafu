use std::collections::HashMap;

use iced::Point;
use petgraph::graph::NodeIndex;


#[derive(Clone)]
pub struct Grid {
    pub step_size: f32,
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
        }
    }

    pub fn get_object_in_world(&self, world: Point) -> Option<&NodeIndex> {
        self.objects.get(&self.to_gridpoint(world))
    }

    // clamps the point to top left point on grid
    pub fn to_grid_tl(&self, world: Point) -> Point {
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
