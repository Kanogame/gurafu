use iced::{mouse, widget::canvas, Color, Point, Rectangle, Renderer, Theme};

use crate::gurafuApp::canvas::grid::Camera;

mod grid;

#[derive(Debug, Clone)]
pub struct CanvasState {
    camera: Camera,
    circle: Circle,
}


#[derive(Debug, Clone)]
pub enum CanvasMessage {
    // messages
}

#[derive(Debug, Clone)]
struct Circle {
    radius: f32,
    pos: Point<f32>,
}

impl<CanvasMessage> canvas::Program<CanvasMessage> for CanvasState {
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
        
        //self.camera.setSize(bounds.size());
        

        // We create a `Path` representing a simple circle
        let worldpos= self.camera.WorldToScreen(self.circle.pos);
        println!("{}, {:?}", worldpos, bounds);
        let circle = canvas::Path::circle(worldpos, self.circle.radius);

        // And fill it with some color
        frame.fill(&circle, Color::BLACK);

        // Then, we produce the geometry
        vec![frame.into_geometry()]

        }
}

impl CanvasState {
    pub fn new() -> Self {
        return CanvasState{
            camera: Camera::new(),
            circle: Circle { radius: 200_f32, pos: Point { x: 0_f32, y: 0_f32 } }
        };
    }

    pub fn view(&self) -> iced::Element<CanvasMessage> {
        canvas(self).into()
    }
}

    