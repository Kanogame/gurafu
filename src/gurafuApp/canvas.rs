use iced::{mouse, widget::canvas, Color, Rectangle, Renderer, Theme};

#[derive(Debug, Clone)]
pub struct CanvasState {
    // some state
}


#[derive(Debug, Clone)]
pub enum CanvasMessage {
    // messages
}

#[derive(Debug)]
struct Circle {
    radius: f32,
}

impl<canvasMessage> canvas::Program<canvasMessage> for Circle {
    type State = ();


    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor) -> Vec<canvas::Geometry> {

         // We prepare a new `Frame`
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // We create a `Path` representing a simple circle
        let circle = canvas::Path::circle(frame.center(), self.radius);

        // And fill it with some color
        frame.fill(&circle, Color::BLACK);

        // Then, we produce the geometry
        vec![frame.into_geometry()]

        }
}

impl CanvasState {
    pub fn new() -> Self {
        return CanvasState{};
    }

    pub fn view(&self) -> iced::Element<CanvasMessage> {
        canvas(Circle { radius: 50.0 }).into()
    }
}

    