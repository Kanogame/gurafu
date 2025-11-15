use std::collections::HashMap;

use petgraph::graph::NodeIndex;

use crate::gurafu_app::canvas::camera::WorldPoint;

#[derive(Clone)]
pub struct Grid {
    pub step_size: i32,
    objects: HashMap<GridPoint, NodeIndex>,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub struct GridPoint {
    pub x: i32,
    pub y: i32,
}

impl std::hash::Hash for GridPoint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl Eq for GridPoint {}

impl Into<WorldPoint> for GridPoint {
    fn into(self) -> WorldPoint {
        WorldPoint {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}

impl Grid {
    pub fn new() -> Self {
        Grid {
            step_size: 100,
            objects: HashMap::new(),
        }
    }

    pub fn get_from_grid(&self, grid: GridPoint) -> Option<&NodeIndex> {
        self.objects.get(&grid)
    }

    // clamps the point to top left point on grid
    pub fn to_grid_tl(&self, world: WorldPoint) -> GridPoint {
        GridPoint {
            x: (world.x / (self.step_size as f32)).floor() as i32 * self.step_size,
            y: (world.y / (self.step_size as f32)).floor() as i32 * self.step_size,
        }
    }

    // claps the point to closest point on grid
    pub fn to_grid(&self, world: WorldPoint) -> GridPoint {
        GridPoint {
            x: (world.x / (self.step_size as f32)).round() as i32 * self.step_size,
            y: (world.y / (self.step_size as f32)).round() as i32 * self.step_size,
        }
    }

    pub fn add_to_grid(&mut self, grid: GridPoint, object: NodeIndex) {
        if !self.objects.contains_key(&grid) {
            self.objects.insert(grid, object);
        }
    }

    pub fn remove_from_grid(&mut self, grid: GridPoint) {
        self.objects.remove(&grid);
    }
}