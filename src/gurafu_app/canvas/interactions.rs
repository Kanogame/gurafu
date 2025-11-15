use iced::{Point, Size, mouse, widget::canvas};

use crate::gurafu_app::{
    canvas::{CanvasMessage, camera::{Camera, WorldPoint}, grid::{Grid, GridPoint}, helpers},
    toolbar::ToolbarOption,
};

pub struct CanvasStateInternal {
    // drag
    is_dragging: bool,
    drag_start_position: Point,
    drag_offset: Point,

    // rendering
    pub points: Vec<canvas::Path>,

    // state
    pub camera: Camera,
    pub toolbar_state: ToolbarOption,
}

impl Default for CanvasStateInternal {
    fn default() -> Self {
        CanvasStateInternal::new()
    }
}

impl CanvasStateInternal {
    pub fn new() -> Self {
        return CanvasStateInternal {
            points: Vec::new(),
            camera: Camera::new(),
            toolbar_state: ToolbarOption::new(),
            is_dragging: false,
            drag_start_position: Point { x: 0_f32, y: 0_f32 },
            drag_offset: Point { x: 0_f32, y: 0_f32 },
        };
    }

    pub fn update_grid_points(&mut self, grid: &Grid, size: Size) {
        // top left corner of screen in world cords
        let camera_tl = self.camera.screen_to_world(Point { x: 0_f32, y: 0_f32 });
        // bottom right corner of screen in world cords
        let camera_br = self.camera.screen_to_world(Point {
            x: size.width,
            y: size.height,
        });

        let mut points: Vec<canvas::Path> = vec![];

        let start_on_grid = grid.to_grid_tl(camera_tl);
        let end_on_grid = grid.to_grid_tl(camera_br);

        let mut x_offset = 0;
        let mut y_offset = 0;

        while start_on_grid.x + x_offset <= end_on_grid.x {
            while start_on_grid.y + y_offset <= end_on_grid.y {
                points.push(canvas::Path::circle(
                    self.camera.world_to_screen(GridPoint {
                        x: start_on_grid.x + x_offset,
                        y: start_on_grid.y + y_offset,
                    }.into()),
                    2_f32 * (1_f32 / self.camera.scale),
                ));

                y_offset += grid.step_size;
            }

            y_offset = 0;
            x_offset += grid.step_size;
        }

        self.points = points;
    }

    pub fn get_grid_points(&self) -> &Vec<canvas::Path> {
        &self.points
    }

    // reset when cursor is out of bounds
    pub fn reset_on_oob(&mut self) {
        self.is_dragging = false;
    }

    pub fn convert_screen_to_world(&self, screen: Point) -> WorldPoint {
        self.camera.screen_to_world(screen)
    }

    pub fn get_cursor_state(&self) -> mouse::Interaction {
        match self.toolbar_state {
            ToolbarOption::Hand => {
                if self.is_dragging {
                    mouse::Interaction::Grabbing
                } else {
                    mouse::Interaction::Grab
                }
            }
            ToolbarOption::Node => mouse::Interaction::Pointer,
            _ => mouse::Interaction::None,
        }
    }

    // left mouse button
    pub fn handle_left_mouse_released(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        match self.toolbar_state {
            ToolbarOption::Hand => {
                if self.is_dragging {
                    self.is_dragging = false;

                    self.camera.apply_drag(self.drag_offset);
                    return helpers::CAPTURED;
                }
            }
            ToolbarOption::Node => {
                self.is_dragging = false;

                if cursor.is_some() {
                    return self.create_new_node_on_grid(cursor.unwrap());
                }
            }
            ToolbarOption::Connection => {
                if cursor.is_some() {
                    return helpers::capured_message(CanvasMessage::HandleConnection(
                        self.camera.screen_to_world(cursor.unwrap()),
                    ));
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
            ToolbarOption::Hand => {
                if !self.is_dragging && cursor.is_some() {
                    self.is_dragging = true;
                    self.drag_start_position = cursor.unwrap();
                    return helpers::CAPTURED;
                }
            }
            ToolbarOption::Node => {
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
            ToolbarOption::Node => {
                if cursor.is_some() {
                    return self.remove_node_from_grid(cursor.unwrap());
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

    fn create_new_node_on_grid(
        &mut self,
        screen: Point,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        helpers::capured_message(CanvasMessage::CreateNodeOnGrid(
            self.camera.screen_to_world(screen),
        ))
    }

    fn remove_node_from_grid(
        &mut self,
        screen: Point,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        helpers::capured_message(CanvasMessage::RemoveNodeFromGrid(
            self.camera.screen_to_world(screen),
        ))
    }
}
