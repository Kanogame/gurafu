use std::fs::File;

use iced::{
    Point,
    alignment::Horizontal,
    widget::{button, column, pane_grid, text},
};

use crate::gurafuApp::canvas::CanvasMessage;

mod canvas;
mod file;

pub struct GurafuApplication {
    panes: pane_grid::State<Pane>,
    file: file::FileState,
    canvas: canvas::CanvasState,
}

#[derive(Debug)]
enum Pane {
    Canvas,
    File,
    Player,
}

#[derive(Debug, Clone)]
enum GurafuMessage {
    PaneResized(pane_grid::ResizeEvent),
    File(file::FileMessage),
    Canvas(canvas::CanvasMessage),
}

pub fn run() -> iced::Result {
    iced::application("Gurafu", GurafuApplication::update, GurafuApplication::view)
        .theme(GurafuApplication::theme)
        .run()
}

impl Default for GurafuApplication {
    fn default() -> Self {
        GurafuApplication::new()
    }
}

impl GurafuApplication {
    fn new() -> Self {
        let panes = pane_grid::State::with_configuration(pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Vertical,
            ratio: 0.2,
            a: Box::new(pane_grid::Configuration::Split {
                axis: pane_grid::Axis::Horizontal,
                ratio: 0.8,
                a: Box::new(pane_grid::Configuration::Pane(Pane::File)),
                b: Box::new(pane_grid::Configuration::Pane(Pane::Player)),
            }),
            b: Box::new(pane_grid::Configuration::Pane(Pane::Canvas)),
        });

        GurafuApplication {
            panes,
            file: file::FileState::new(),
            canvas: canvas::CanvasState::new(),
        }
    }

    fn update(&mut self, message: GurafuMessage) {
        match message {
            GurafuMessage::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
            }
            GurafuMessage::File(some) => {}
            GurafuMessage::Canvas(ev) => match ev {
                CanvasMessage::DraggingStart(pos) => {
                    self.canvas.drag_start_position = pos;
                }
                CanvasMessage::DraggingEnd => {
                    self.canvas.is_dragging = false;

                    self.canvas
                        .camera
                        .applyDragPosition(self.canvas.drag_offset);
                }
                CanvasMessage::Dragging(pos) => {
                    let drag_start = self.canvas.drag_start_position;
                    self.canvas.drag_start_position = pos;

                    self.canvas.drag_offset = Point {
                        x: drag_start.x - pos.x,
                        y: drag_start.y - pos.y,
                    };

                    self.canvas
                        .camera
                        .applyDragPosition(self.canvas.drag_offset);
                }
                _ => {}
            },
        }
    }

    fn view(&self) -> iced::Element<GurafuMessage> {
        pane_grid(&self.panes, |pane, state, is_maximized| match state {
            Pane::File => pane_grid::Content::new({
                file::FileState::view(&self.file).map(GurafuMessage::File)
            }),
            Pane::Canvas => pane_grid::Content::new({
                canvas::CanvasState::view(&self.canvas).map(GurafuMessage::Canvas)
            }),
            Pane::Player => pane_grid::Content::new({ text("this will be a player") }),
        })
        .on_resize(10, GurafuMessage::PaneResized)
        .into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}
// a sidebar with:
// 1. file open prompt
// 2. file save prompt
// 3. player thingies

// a canvas with:
// 1. inifinite grid
// 2. dragging
// 3. graphs
