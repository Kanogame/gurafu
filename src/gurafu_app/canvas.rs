use iced::{
    mouse::{self, ScrollDelta}, widget::{self, canvas::{self, Path}}, Color, Point, Rectangle, Renderer, Size, Theme, Vector
};

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
        state.grid.update_grid_points(&state.camera, bounds.size());

        match event {
            // although implementing state mutation in a canvas defies Elm arch,
            // propagating it higher would be a giant overhead, so we act
            // like a widget here and handle our state internally, exposing only
            // minimal info needed (currently none)
            canvas::Event::Mouse(m_ev) => match m_ev {
                mouse::Event::ButtonPressed(mouse::Button::Left) => match state.toolbar_state {
                    ToolbarOptions::Hand => {
                        if !state.is_dragging && pos.is_some() {
                            state.is_dragging = true;
                            state.drag_start_position = pos.unwrap();
                            (canvas::event::Status::Captured, None)
                        } else {
                            (canvas::event::Status::Ignored, None)
                        }
                    }
                    ToolbarOptions::Node => {
                        state.is_dragging = false;
                        (canvas::event::Status::Ignored, None)
                    }
                    _ => (canvas::event::Status::Ignored, None),
                },
                mouse::Event::ButtonPressed(mouse::Button::Middle) => {
                        if !state.is_dragging && pos.is_some() {
                            state.is_dragging = true;
                            state.drag_start_position = pos.unwrap();
                            (canvas::event::Status::Captured, None)
                        } else {
                            (canvas::event::Status::Ignored, None)
                        }
                },
                mouse::Event::ButtonReleased(button) => match button {
                    mouse::Button::Left => match state.toolbar_state {
                        ToolbarOptions::Hand => {
                            if state.is_dragging {
                                state.is_dragging = false;

                                state.camera.apply_drag(state.drag_offset);
                                (canvas::event::Status::Captured, None)
                            } else {
                                (canvas::event::Status::Ignored, None)
                            }
                        }
                        ToolbarOptions::Node => {
                            state.is_dragging = false;

                            if pos.is_some() {
                                state.create_new_node_on_grid(pos.unwrap());

                                (canvas::event::Status::Captured, None)
                            } else {
                                (canvas::event::Status::Ignored, None)
                            }
                        }
                        ToolbarOptions::Connection => {
                            if pos.is_some() {
                                if state.is_connecting {
                                    // some
                                     (canvas::event::Status::Ignored, None)
                                } else {
                                    state.is_connecting = true;
                                    state.connection_start = state.grid.to_grid(state.camera.screen_to_world( pos.unwrap()));

                                    (canvas::event::Status::Captured, None)
                                }
                            } else {
                                (canvas::event::Status::Ignored, None)
                            }
                        }
                    },
                    mouse::Button::Right => match state.toolbar_state {
                        ToolbarOptions::Node => {
                            if pos.is_some() {
                                state.remove_node_from_grid( pos.unwrap());

                                (canvas::event::Status::Captured, None)
                            } else {
                                (canvas::event::Status::Ignored, None)
                            }
                        }
                        _ => (canvas::event::Status::Ignored, None),
                    },
                    mouse::Button::Middle => {
                        if state.is_dragging {
                                state.is_dragging = false;

                                state.camera.apply_drag(state.drag_offset);
                                (canvas::event::Status::Captured, None)
                            } else {
                                (canvas::event::Status::Ignored, None)
                            }
                    },
                    _ => (canvas::event::Status::Ignored, None),
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
                        (canvas::event::Status::Ignored, None)
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
                    
                    return (canvas::event::Status::Captured, None);
                }
                _ => (canvas::event::Status::Ignored, None),
            },
            _ => (canvas::event::Status::Ignored, None),
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

        // fill objects on canvas
        for (pos, obj) in state.grid.objects() {
            frame.fill(
                &obj.into_path(
                    Point {
                        x: pos.0.x as f32,
                        y: pos.0.y as f32,
                    },
                    &state.camera,
                ),
                theme.palette().primary,
            );
        }

        if state.is_connecting {
            let point = state.camera.world_to_screen(state.connection_start);
            let cursor = cursor.position_in(bounds).unwrap();

            let connection = Arrow{
                start: point,
                end: cursor,
                line_width: 10.0,
                arrowhead_size: 30.0,
            };
            
            
            Path::rectangle(point, Size{
                width: cursor.x - point.x,
                height: cursor.y - point.y,
            });
            frame.fill(&connection.into_path(Point { x: 0.0, y: 0.0 },  &state.camera), Color{r: 1.0, g: 1.0, b: 1.0, a: 1.0});
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
            connection_start: Point { x: 0_f32, y: 0_f32 }
        };
    }

    fn create_new_node_on_grid(&mut self, screen: Point) {
        let object = Circle { radius: 30_f32 };

        self.grid
            .add_to_grid(self.camera.screen_to_world(screen), Box::new(object))
    }

    fn remove_node_from_grid(&mut self, screen: Point) {

        self.grid
            .remove_from_grid(self.camera.screen_to_world(screen))
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
