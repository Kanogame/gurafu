
use iced::time::{self, Duration};


use iced::{widget::pane_grid, Settings, Subscription};

use crate::gurafu_app::{canvas::CanvasMessage, player::PlayerMessage, toolbar::ToolbarMessage};

mod canvas;
mod file;
mod player;
mod styles;
mod toolbar;

pub struct GurafuApplication {
    panes: pane_grid::State<Pane>,
    file: file::FileState,
    canvas: canvas::CanvasState,
    toolbar: toolbar::ToolbarState,
    player: player::PlayerState,
}

#[derive(Debug)]
enum Pane {
    Canvas,
    File,
    Player,
    Toolbar,
}

#[derive(Debug, Clone)]
enum GurafuMessage {
    PaneResized(pane_grid::ResizeEvent),
    File(file::FileMessage),
    Canvas(canvas::CanvasMessage),
    Toolbar(toolbar::ToolbarMessage),
    Player(player::PlayerMessage),

    AlgorithmTick,
}

pub fn run() -> iced::Result {
    iced::application("Gurafu", GurafuApplication::update, GurafuApplication::view)
        .theme(GurafuApplication::theme)
        .settings(Settings {
            antialiasing: true,
            ..Settings::default()
        })
        .subscription(GurafuApplication::subscription)
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
            b: Box::new(pane_grid::Configuration::Split {
                axis: pane_grid::Axis::Horizontal,
                ratio: 0.05,
                a: Box::new(pane_grid::Configuration::Pane(Pane::Toolbar)),
                b: Box::new(pane_grid::Configuration::Pane(Pane::Canvas)),
            }),
        });

        GurafuApplication {
            panes,
            file: file::FileState::new(),
            canvas: canvas::CanvasState::new(),
            toolbar: toolbar::ToolbarState::new(),
            player: player::PlayerState::new(),
        }
    }

    fn update(state: &mut GurafuApplication, message: GurafuMessage) {
        match message {
            GurafuMessage::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                state.panes.resize(split, ratio);
            }
            GurafuMessage::File(_) => {}
            GurafuMessage::Canvas(message) => match message {
                CanvasMessage::SolveFlueryResponce => {
                    state.canvas.solving_required = false;
                    //match resp {
                    //    Some(vec) => {
                    //        println!("the Euler's circuit is: {:?}", vec)
                    //    }
                    //    None => {
                    //        println!("there is no Euler's circuit in following graph")
                    //    }
                    //};
                }
            },
            GurafuMessage::Toolbar(message) => match message {
                ToolbarMessage::ChosenState(new_state) => {
                    state.toolbar.state = new_state.clone();
                    state.canvas.toolbar_state = new_state;
                }
            },
            GurafuMessage::Player(message) => match message {
                PlayerMessage::PlayPause => {
                    state.player.playing = !state.player.playing;
                    //state.canvas.solving_required = true;
                }
                _ => {}
            },
            GurafuMessage::AlgorithmTick => {
                state.canvas.solving_required = true;
            }
        }
    }

    fn subscription(state: &GurafuApplication) -> Subscription<GurafuMessage> {
        if state.player.playing {
            time::every(Duration::from_secs(1)).map(|_| GurafuMessage::AlgorithmTick)
        } else {
            Subscription::none()
        }
    }

    fn view(state: &GurafuApplication) -> iced::Element<'_, GurafuMessage> {
        pane_grid(&state.panes, |_, pane_state, _| match pane_state {
            Pane::File => pane_grid::Content::new({
                file::FileState::view(&state.file).map(GurafuMessage::File)
            })
            .style(styles::pane_grid_style),
            Pane::Canvas => pane_grid::Content::new({
                canvas::CanvasState::view(&state.canvas).map(GurafuMessage::Canvas)
            })
            .style(styles::pane_grid_style),
            Pane::Player => pane_grid::Content::new({
                player::PlayerState::view(&state.player).map(GurafuMessage::Player)
            })
            .style(styles::pane_grid_style),
            Pane::Toolbar => pane_grid::Content::new({
                toolbar::ToolbarState::view(&state.toolbar).map(GurafuMessage::Toolbar)
            })
            .style(styles::pane_grid_style),
        })
        .on_resize(10, GurafuMessage::PaneResized)
        .into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}
