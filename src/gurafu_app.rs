use iced::widget::{pane_grid, text};

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

    fn update(state: &mut GurafuApplication, message: GurafuMessage) {
        match message {
            GurafuMessage::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                state.panes.resize(split, ratio);
            }
            GurafuMessage::File(_) => {}
            GurafuMessage::Canvas(_) => {}
        }
    }

    fn view(state: &GurafuApplication) -> iced::Element<'_, GurafuMessage> {
        pane_grid(&state.panes, |_, pane_state, _| match pane_state {
            Pane::File => pane_grid::Content::new({
                file::FileState::view(&state.file).map(GurafuMessage::File)
            }),
            Pane::Canvas => pane_grid::Content::new({
                canvas::CanvasState::view(&state.canvas).map(GurafuMessage::Canvas)
            }),
            Pane::Player => pane_grid::Content::new(text("this will be a player")),
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
