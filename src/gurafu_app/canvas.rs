use iced::{
    Point, Rectangle, Renderer, Theme, Vector,
    mouse::{self, ScrollDelta},
    widget::{canvas, combo_box::State},
};

use crate::gurafu_app::{
    canvas::{
        camera::Camera,
        drawable::{Circle, Drawable},
        grid::Grid,
    },
    toolbar::{self, ToolbarOptions},
};

mod camera;
mod drawable;
mod grid;

pub struct CanvasStateInternal {
    is_dragging: bool,
    drag_start_position: Point,
    drag_offset: Point,
    camera: Camera,
    grid: Grid,
    objects: Vec<Box<dyn Drawable>>,
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
        _bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        state.toolbar_state = self.toolbar_state.clone();
        match event {
            // although implementing state mutation in a canvas defies Elm arch,
            // propagating it higher would be a giant overhead, so we act
            // like a widget here and handle our state internally, exposing only
            // minimal info needed
            canvas::Event::Mouse(m_ev) => match m_ev {
                mouse::Event::ButtonPressed(mouse::Button::Left) => match state.toolbar_state {
                    ToolbarOptions::Hand => {
                        let pos = cursor.position();
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
                mouse::Event::ButtonReleased(mouse::Button::Left) => match state.toolbar_state {
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

                        state.create_new_node_on_grid(cursor.position().unwrap());

                        (canvas::event::Status::Captured, None)
                    }
                    _ => (canvas::event::Status::Ignored, None),
                },
                mouse::Event::CursorMoved { position } => {
                    if state.is_dragging {
                        let drag_start = state.drag_start_position;
                        state.drag_start_position = position;
                        state.drag_offset = Point {
                            x: drag_start.x - position.x,
                            y: drag_start.y - position.y,
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

                    state.camera.apply_scroll(delta_vec.x);

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
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // We prepare a new `Frame`
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        //self.camera.setSize(bounds.size());

        let points = state.grid.get_grid_points(&state.camera, bounds.size());

        // fill grid points (alignment helpers)
        for point in points {
            frame.fill(&point, theme.palette().success);
        }

        // fill objects on canvas
        for obj in state.objects.iter() {
            frame.fill(&obj.into_path(&state.camera), theme.palette().primary);
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
        if state.is_dragging {
            mouse::Interaction::Grabbing
        } else {
            mouse::Interaction::Grab
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
            objects: vec![],
        };
    }

    fn create_new_node_on_grid(&mut self, screen: Point) {
        let world_grid = self.grid.to_grid(self.camera.scree_to_world(screen));
        let object = Circle {
            radius: 30_f32,
            pos: world_grid,
        };

        self.objects.push(Box::new(object));
    }
}

impl CanvasState {
    pub fn new() -> Self {
        CanvasState {
            toolbar_state: ToolbarOptions::new(),
        }
    }

    pub fn view(state: &CanvasState) -> iced::Element<'_, CanvasMessage> {
        canvas(state).into()
    }
}
