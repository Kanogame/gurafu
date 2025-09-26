use iced::{Point, mouse, widget::canvas};
use petgraph::prelude::StableGraph;

use crate::gurafu_app::{
    canvas::{
        CanvasMessage,
        camera::Camera,
        canvas_internal::grid::Grid,
        drawable::{Arrow, Circle},
        helpers,
    },
    toolbar::ToolbarOptions,
};

mod grid;

pub struct CanvasStateInternal {
    // drag
    is_dragging: bool,
    drag_start_position: Point,
    drag_offset: Point,

    // connection
    is_connecting: bool,
    connection_start: Point,

    // state
    pub camera: Camera,
    pub grid: Grid,
    pub toolbar_state: ToolbarOptions,
    pub graph: StableGraph<Circle, Arrow>,
}

impl Default for CanvasStateInternal {
    fn default() -> Self {
        CanvasStateInternal::new()
    }
}

impl CanvasStateInternal {
    pub fn new() -> Self {
        return CanvasStateInternal {
            camera: Camera::new(),
            grid: Grid::new(),
            toolbar_state: ToolbarOptions::new(),
            is_dragging: false,
            drag_start_position: Point { x: 0_f32, y: 0_f32 },
            drag_offset: Point { x: 0_f32, y: 0_f32 },
            is_connecting: false,
            connection_start: Point { x: 0_f32, y: 0_f32 },
            graph: StableGraph::new(),
        };
    }

    // reset when cursor is out of bounds
    pub fn reset_on_oob(&mut self) {
        self.is_connecting = false;
        self.is_dragging = false;
    }

    pub fn draw_arrow(&self, screen: Point) -> Option<Arrow> {
        if self.is_connecting {
            Some(Arrow {
                start: self.connection_start,
                end: self.camera.screen_to_world(screen),
                line_width: 10.0,
                arrowhead_size: 30.0,
                offset: 0.0,
            });
        }
        None
    }

    pub fn get_cursor_state(&self) -> mouse::Interaction {
        match self.toolbar_state {
            ToolbarOptions::Hand => {
                if self.is_dragging {
                    mouse::Interaction::Grabbing
                } else {
                    mouse::Interaction::Grab
                }
            }
            ToolbarOptions::Node => mouse::Interaction::Pointer,
            _ => mouse::Interaction::None,
        }
    }

    // left mouse button
    pub fn handle_left_mouse_released(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        match self.toolbar_state {
            ToolbarOptions::Hand => {
                if self.is_dragging {
                    self.is_dragging = false;

                    self.camera.apply_drag(self.drag_offset);
                    return helpers::CAPTURED;
                }
            }
            ToolbarOptions::Node => {
                self.is_dragging = false;

                if cursor.is_some() {
                    self.create_new_node_on_grid(cursor.unwrap());
                    return helpers::CAPTURED;
                }
            }
            ToolbarOptions::Connection => {
                if cursor.is_some() {
                    if self.is_connecting {
                        self.end_connection(cursor.unwrap());
                        return helpers::CAPTURED;
                    }

                    self.start_connection(cursor.unwrap());
                    return helpers::CAPTURED;
                }
            }
        }
        helpers::IGNORED
    }

    pub fn handle_left_mouse_pressed(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        match self.toolbar_state {
            ToolbarOptions::Hand => {
                if !self.is_dragging && cursor.is_some() {
                    self.is_dragging = true;
                    self.drag_start_position = cursor.unwrap();
                    return helpers::CAPTURED;
                }
            }
            ToolbarOptions::Node => {
                self.is_dragging = false;
            }
            _ => {}
        };
        helpers::IGNORED
    }

    // middle mouse button
    pub fn handle_middle_mouse_pressed(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        if !self.is_dragging && cursor.is_some() {
            self.is_dragging = true;
            self.drag_start_position = cursor.unwrap();
            helpers::CAPTURED
        } else {
            helpers::IGNORED
        }
    }

    pub fn handle_middle_mouse_released(
        &mut self,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        if self.is_dragging {
            self.is_dragging = false;

            self.camera.apply_drag(self.drag_offset);
            helpers::CAPTURED
        } else {
            helpers::IGNORED
        }
    }

    // right mouse button
    pub fn handle_right_mouse_release(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        match self.toolbar_state {
            ToolbarOptions::Node => {
                if cursor.is_some() {
                    self.remove_node_from_grid(cursor.unwrap());

                    return helpers::CAPTURED;
                }
            }
            _ => {}
        };
        helpers::IGNORED
    }

    pub fn handle_mouse_moved(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        if self.is_dragging && cursor.is_some() {
            let drag_start: Point = self.drag_start_position;
            self.drag_start_position = cursor.unwrap();
            self.drag_offset = Point {
                x: drag_start.x - cursor.unwrap().x,
                y: drag_start.y - cursor.unwrap().y,
            };

            self.camera.apply_drag(self.drag_offset);
            helpers::CAPTURED
        } else {
            helpers::IGNORED
        }
    }

    fn create_new_node_on_grid(&mut self, screen: Point) {
        let grid_pos = self.grid.to_grid(self.camera.screen_to_world(screen));

        let object = Circle {
            center: grid_pos,
            radius: 30_f32,
        };

        match self.grid.get_object_in_world(grid_pos) {
            Some(_) => {}
            None => {
                let node_index = self.graph.add_node(object);

                self.grid.add_to_grid(grid_pos, node_index);
            }
        }
    }

    fn remove_node_from_grid(&mut self, screen: Point) {
        let grid_pos = self.grid.to_grid(self.camera.screen_to_world(screen));

        let obj = self.grid.get_object_in_world(grid_pos);

        match obj.cloned() {
            // if object is present on grid
            Some(idx) => {
                match self.graph.node_weight(idx) {
                    // if object also present in graph
                    Some(_) => {
                        // remove node itself
                        self.grid.remove_from_grid(grid_pos);

                        self.graph.remove_node(idx.clone());

                        // remove edges
                    }
                    None => {
                        // odd
                        println!("Error: object is not present in grid");

                        self.grid
                            .remove_from_grid(self.camera.screen_to_world(screen));
                    }
                }
            }
            None => {}
        }
    }

    fn start_connection(&mut self, screen: Point) {
        let grid_point = self.grid.to_grid(self.camera.screen_to_world(screen));

        let start_node = self.grid.get_object_in_world(grid_point);

        match start_node {
            Some(_) => {
                self.is_connecting = true;
                self.connection_start = grid_point;
            }
            None => {}
        }
    }

    fn end_connection(&mut self, screen: Point) {
        let start_node = self.grid.get_object_in_world(self.connection_start);
        let end_node = self
            .grid
            .get_object_in_world(self.camera.screen_to_world(screen));

        if start_node.is_some() && end_node.is_some() {
            self.graph.add_edge(
                start_node.unwrap().clone(),
                end_node.unwrap().clone(),
                Arrow {
                    start: self.grid.to_grid(self.connection_start),
                    end: self.grid.to_grid(self.camera.screen_to_world(screen)),
                    line_width: 5.0,
                    arrowhead_size: 10.0,
                    offset: 30.0,
                },
            );
        }

        self.is_connecting = false;
    }
}
