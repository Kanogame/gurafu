use iced::{
    Color, Event, Point, Rectangle, Renderer, Theme,
    alignment::Horizontal::{Left, Right},
    mouse,
    widget::{canvas, pane_grid::Draggable, text_input::cursor},
};

use crate::gurafuApp::canvas::grid::Camera;

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
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if !state.is_dragging && cursor.position().is_some() {
                    state.is_dragging = true;
                    (
                        canvas::event::Status::Captured,
                        Some(CanvasMessage::DraggingStart(cursor.position().unwrap())),
                    )
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.is_dragging {
                    state.is_dragging = false;
                    (
                        canvas::event::Status::Captured,
                        Some(CanvasMessage::DraggingEnd),
                    )
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if state.is_dragging {
                    (
                        canvas::event::Status::Captured,
                        Some(CanvasMessage::Dragging(position)),
                    )
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // We prepare a new `Frame`
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        //self.camera.setSize(bounds.size());

        // We create a `Path` representing a simple circle
        let worldpos = self.camera.WorldToScreen(self.circle.pos);
        let circle = canvas::Path::circle(worldpos, self.circle.radius);

        // And fill it with some color
        frame.fill(&circle, Color::BLACK);

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
