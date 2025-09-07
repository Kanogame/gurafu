use iced::{
    Color, Event, Point, Rectangle, Renderer, Theme, Vector,
    alignment::Horizontal::{Left, Right},
    mouse::{self, ScrollDelta},
    widget::{canvas, pane_grid::Draggable, text_input::cursor},
};

use crate::gurafuApp::canvas::camera::Camera;

mod camera;
mod grid;

#[derive(Debug, Default, Clone)]
pub struct CanvasState {
    pub is_dragging: bool,
    pub drag_start_position: Point,
    pub drag_offset: Point,
    pub camera: Camera,
    circle: Circle,
}

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    DraggingStart(Point),
    DraggingEnd,
    Dragging(Point),
    Scroll(Vector),
}

#[derive(Debug, Default, Clone)]
struct Circle {
    radius: f32,
    pos: Point<f32>,
}

impl canvas::Program<CanvasMessage> for CanvasState {
    type State = CanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        _bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        match event {
            // although implementing state mutation in a canvas defies Elm arch,
            // propagating it higher would be a giant overhead, so we act
            // like a widget here and handle our state internally, exposing only
            // minimal info needed
            canvas::Event::Mouse(m_ev) => match m_ev {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    let pos = cursor.position();
                    if !state.is_dragging && pos.is_some() {
                        state.is_dragging = true;
                        state.drag_start_position = pos.unwrap();
                        (canvas::event::Status::Captured, None)
                    } else {
                        (canvas::event::Status::Ignored, None)
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if state.is_dragging {
                        state.is_dragging = false;

                        state.camera.applyDragPosition(self.drag_offset);
                        (canvas::event::Status::Captured, None)
                    } else {
                        (canvas::event::Status::Ignored, None)
                    }
                }
                mouse::Event::CursorMoved { position } => {
                    if state.is_dragging {
                        let drag_start = state.drag_start_position;
                        state.drag_start_position = position;
                        state.drag_offset = Point {
                            x: drag_start.x - position.x,
                            y: drag_start.y - position.y,
                        };

                        state.camera.applyDragPosition(state.drag_offset);
                        (canvas::event::Status::Captured, None)
                    } else {
                        (canvas::event::Status::Ignored, None)
                    }
                }
                mouse::Event::WheelScrolled { delta } => {
                    let deltaVector: Vector = match delta {
                        ScrollDelta::Lines { x, y } => Vector { x: x, y: y },
                        ScrollDelta::Pixels { x, y } => Vector { x: x, y: y },
                    };

                    state.camera.applyScroll(deltaVector.x);

                    return (canvas::event::Status::Captured, None);
                }
                _ => (canvas::event::Status::Ignored, None),
            },
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // We prepare a new `Frame`
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        //self.camera.setSize(bounds.size());

        // We create a `Path` representing a simple circle
        let worldpos = self.camera.WorldToScreen(self.circle.pos);
        let circle: canvas::Path = canvas::Path::circle(worldpos, self.circle.radius);

        // And fill it with some color
        frame.fill(&circle, theme.palette().primary);

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

impl CanvasState {
    pub fn new() -> Self {
        return CanvasState {
            camera: Camera::new(),
            is_dragging: false,
            drag_start_position: Point { x: 0_f32, y: 0_f32 },
            drag_offset: Point { x: 0_f32, y: 0_f32 },
            circle: Circle {
                radius: 200_f32,
                pos: Point { x: 0_f32, y: 0_f32 },
            },
        };
    }

    pub fn view(&self) -> iced::Element<CanvasMessage> {
        canvas(self).into()
    }
}
