use iced::{
    Color, Point, Rectangle, Renderer, Size, Theme, Vector,
    mouse::{self, ScrollDelta},
    widget::{
        self,
        canvas::{self, Path},
    },
};
use petgraph::Graph;

use crate::gurafu_app::{
    canvas::{
        camera::Camera,
        drawable::{Arrow, Circle, Drawable},
        grid::Grid,
    },
    toolbar::ToolbarOptions,
};

mod camera;
mod drawable;
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
    camera: Camera,
    grid: Grid,
    toolbar_state: ToolbarOptions,
    graph: Graph<Circle, Arrow>,
}

#[derive(Clone)]
pub struct CanvasState {
    pub toolbar_state: ToolbarOptions,
}

#[derive(Debug, Clone)]
pub enum CanvasMessage {}

impl canvas::Program<CanvasMessage> for CanvasState {
    type State = CanvasStateInternal;

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        state.toolbar_state = self.toolbar_state.clone();
        let pos = cursor.position_in(bounds);

        if pos.is_none() {
            state.is_connecting = false;
            state.is_dragging = false;
        }

        state.grid.update_grid_points(&state.camera, bounds.size());

        match event {
            // although implementing state mutation in a canvas defies Elm arch,
            // propagating it higher would be a giant overhead, so we act
            // like a widget here and handle our state internally, exposing only
            // minimal info needed (currently none)
            //
            // Btw, this is the "inteded way"
            canvas::Event::Mouse(m_ev) => match m_ev {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    state.handle_left_mouse_pressed(pos)
                }
                mouse::Event::ButtonPressed(mouse::Button::Middle) => {
                    state.handle_middle_mouse_pressed(pos)
                }
                mouse::Event::ButtonReleased(button) => match button {
                    mouse::Button::Left => state.handle_left_mouse_released(pos),
                    mouse::Button::Right => state.handle_right_mouse_release(pos),
                    mouse::Button::Middle => state.handle_middle_mouse_released(),
                    _ => IGNORED,
                },
                mouse::Event::CursorMoved { position: _ } => {
                    if state.is_dragging && pos.is_some() {
                        let drag_start = state.drag_start_position;
                        state.drag_start_position = pos.unwrap();
                        state.drag_offset = Point {
                            x: drag_start.x - pos.unwrap().x,
                            y: drag_start.y - pos.unwrap().y,
                        };

                        state.camera.apply_drag(state.drag_offset);
                        (canvas::event::Status::Captured, None)
                    } else {
                        IGNORED
                    }
                }
                mouse::Event::WheelScrolled { delta } => {
                    let delta_vec: Vector = match delta {
                        ScrollDelta::Lines { x, y } => Vector { x: x, y: y },
                        ScrollDelta::Pixels { x, y } => Vector { x: x, y: y },
                    };

                    println!("{:?}", delta_vec.y);

                    state.camera.apply_scroll(delta_vec.y / 4_f32);
                    println!("{:?}", state.camera);

                    return CAPTURED;
                }
                _ => IGNORED,
            },
            _ => IGNORED,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // We prepare a new `Frame`
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // fill grid points (alignment helpers)
        for point in state.grid.get_grid_points() {
            frame.fill(point, theme.palette().success);
        }

        // fill nodes on canvas
        for node in state.graph.raw_nodes() {
            frame.fill(
                &node.weight.into_path(&state.camera),
                theme.palette().primary,
            );
        }

        // fill edges on canvas
        for edge in state.graph.raw_edges() {
            frame.fill(
                &edge.weight.into_path(&state.camera),
                theme.palette().primary,
            );
        }

        if state.is_connecting {
            match cursor.position_in(bounds) {
                Some(c) => {
                    let connection = Arrow {
                        start: state.connection_start,
                        end: state.camera.screen_to_world(c),
                        line_width: 10.0,
                        arrowhead_size: 30.0,
                    };
                    frame.fill(
                        &connection.into_path(&state.camera),
                        Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        },
                    );
                }
                None => {
                    // cursor is outside the canvas, do nothing
                }
            }
        }

        // Then, we produce the geometry
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        match state.toolbar_state {
            ToolbarOptions::Hand => {
                if state.is_dragging {
                    mouse::Interaction::Grabbing
                } else {
                    mouse::Interaction::Grab
                }
            }
            ToolbarOptions::Node => mouse::Interaction::Pointer,
            _ => mouse::Interaction::None,
        }
    }
}

impl Default for CanvasStateInternal {
    fn default() -> Self {
        CanvasStateInternal::new()
    }
}

const IGNORED: (canvas::event::Status, Option<CanvasMessage>) =
    (canvas::event::Status::Ignored, None::<CanvasMessage>);
const CAPTURED: (canvas::event::Status, Option<CanvasMessage>) =
    (canvas::event::Status::Captured, None::<CanvasMessage>);

impl CanvasStateInternal {
    fn new() -> Self {
        return CanvasStateInternal {
            camera: Camera::new(),
            grid: Grid::new(),
            toolbar_state: ToolbarOptions::new(),
            is_dragging: false,
            drag_start_position: Point { x: 0_f32, y: 0_f32 },
            drag_offset: Point { x: 0_f32, y: 0_f32 },
            is_connecting: false,
            connection_start: Point { x: 0_f32, y: 0_f32 },
            graph: Graph::new(),
        };
    }

    // left mouse button
    fn handle_left_mouse_released(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        match self.toolbar_state {
            ToolbarOptions::Hand => {
                if self.is_dragging {
                    self.is_dragging = false;

                    self.camera.apply_drag(self.drag_offset);
                    return CAPTURED;
                }
            }
            ToolbarOptions::Node => {
                self.is_dragging = false;

                if cursor.is_some() {
                    self.create_new_node_on_grid(cursor.unwrap());
                    return CAPTURED;
                }
            }
            ToolbarOptions::Connection => {
                if cursor.is_some() {
                    if self.is_connecting {
                        self.end_connection(cursor.unwrap());
                        return CAPTURED;
                    }

                    self.start_connection(cursor.unwrap());
                    return CAPTURED;
                }
            }
        }
        IGNORED
    }

    fn handle_left_mouse_pressed(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        match self.toolbar_state {
            ToolbarOptions::Hand => {
                if !self.is_dragging && cursor.is_some() {
                    self.is_dragging = true;
                    self.drag_start_position = cursor.unwrap();
                    return CAPTURED;
                }
            }
            ToolbarOptions::Node => {
                self.is_dragging = false;
            }
            _ => {}
        };
        IGNORED
    }

    // middle mouse button
    fn handle_middle_mouse_pressed(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        if !self.is_dragging && cursor.is_some() {
            self.is_dragging = true;
            self.drag_start_position = cursor.unwrap();
            CAPTURED
        } else {
            IGNORED
        }
    }

    fn handle_middle_mouse_released(&mut self) -> (canvas::event::Status, Option<CanvasMessage>) {
        if self.is_dragging {
            self.is_dragging = false;

            self.camera.apply_drag(self.drag_offset);
            CAPTURED
        } else {
            IGNORED
        }
    }

    // right mouse button
    fn handle_right_mouse_release(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        match self.toolbar_state {
            ToolbarOptions::Node => {
                if cursor.is_some() {
                    self.remove_node_from_grid(cursor.unwrap());

                    return CAPTURED;
                }
            }
            _ => {}
        };
        IGNORED
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
        let obj = self
            .grid
            .get_object_in_world(self.camera.screen_to_world(screen));

        match obj.cloned() {
            // if object is present on grid
            Some(idx) => {
                match self.graph.raw_nodes().get(idx.index()) {
                    // if object also present in graph
                    Some(_) => {
                        // remove node itself
                        self.grid
                            .remove_from_grid(self.camera.screen_to_world(screen));

                        self.graph.remove_node(idx.clone());

                        // remove edges
                    }
                    None => {
                        // odd
                        println!("Error: object is not present in graph");

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
                },
            );
        }

        self.is_connecting = false;
    }
}

impl CanvasState {
    pub fn new() -> Self {
        CanvasState {
            toolbar_state: ToolbarOptions::new(),
        }
    }

    pub fn view(state: &CanvasState) -> iced::Element<'_, CanvasMessage> {
        widget::canvas(state).into()
    }
}
