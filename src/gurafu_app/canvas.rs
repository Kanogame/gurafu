use iced::{
    Color, Rectangle, Renderer, Theme, Vector,
    mouse::{self, ScrollDelta},
    widget::{self, canvas},
};
use petgraph::visit::{IntoEdgeReferences, IntoNodeReferences};

use crate::gurafu_app::{canvas::drawable::Drawable, toolbar::ToolbarOptions};

mod camera;
mod canvas_internal;
mod drawable;
mod helpers;

#[derive(Clone)]
pub struct CanvasState {
    pub toolbar_state: ToolbarOptions,
    pub solving_required: bool,
}

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    SolveFlurry,
}

impl canvas::Program<CanvasMessage> for CanvasState {
    type State = canvas_internal::CanvasStateInternal;

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        state.toolbar_state = self.toolbar_state.clone();
        let pos = cursor.position_in(bounds);

        if self.solving_required {
            println!("{:?}", state.solve_flurry());
        }

        if pos.is_none() {
            state.reset_on_oob();
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
                    _ => helpers::IGNORED,
                },
                mouse::Event::CursorMoved { position: _ } => state.handle_mouse_moved(pos),
                mouse::Event::WheelScrolled { delta } => {
                    let delta_vec: Vector = match delta {
                        ScrollDelta::Lines { x, y } => Vector { x: x, y: y },
                        ScrollDelta::Pixels { x, y } => Vector { x: x, y: y },
                    };

                    state.camera.apply_scroll(delta_vec.y / 4_f32);
                    state.grid.update_grid_points(&state.camera, bounds.size());

                    return helpers::CAPTURED;
                }
                _ => helpers::IGNORED,
            },
            _ => helpers::IGNORED,
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
        for (_, crcl) in state.graph.node_references() {
            frame.fill(&crcl.into_path(&state.camera), theme.palette().primary);
        }

        // fill edges on canvas
        for edge in state.graph.edge_references() {
            frame.fill(
                &edge.weight().into_path(&state.camera),
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
            );
        }

        match cursor.position_in(bounds) {
            Some(c) => match state.draw_arrow(c) {
                Some(arrow) => {
                    frame.fill(
                        &arrow.into_path(&state.camera),
                        Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        },
                    );
                }
                None => {}
            },
            None => {
                // cursor is outside the canvas, do nothing
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
        state.get_cursor_state()
    }
}

impl CanvasState {
    pub fn new() -> Self {
        CanvasState {
            toolbar_state: ToolbarOptions::new(),
            solving_required: false,
        }
    }

    pub fn view(state: &CanvasState) -> iced::Element<'_, CanvasMessage> {
        widget::canvas(state).into()
    }

    pub fn update(&mut self, _: CanvasMessage) {
        self.solving_required = !self.solving_required;
    }
}
