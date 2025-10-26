use iced::time::{self};

use iced::widget::text;
use iced::{Settings, Subscription, widget::pane_grid};

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

    show_completed_modal: bool,
    completion_successed: bool,
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

            show_completed_modal: false,
            completion_successed: false,
        }
    }

    fn update(state: &mut GurafuApplication, message: GurafuMessage) {
        match message {
            GurafuMessage::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                state.panes.resize(split, ratio);
            }
            GurafuMessage::File(_) => {}
            GurafuMessage::Canvas(message) => match message {
                CanvasMessage::CreateNodeOnGrid(world) => {
                    state.canvas.create_new_node_on_grid(world);
                }
                CanvasMessage::RemoveNodeFromGrid(world) => {
                    state.canvas.remove_node_from_grid(world);
                }
                CanvasMessage::HandleConnection(world) => {
                    state.canvas.handle_connection(world);
                }
                _ => {}
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
                }
                PlayerMessage::Restart => {
                    state.canvas.reset_algorithm();
                    state.player.playing = false;
                }
                PlayerMessage::NextStep => match state.canvas.step_algorithm() {
                    Some(mes) => match mes {
                        CanvasMessage::AlgorithmFinished(res) => {
                            state.show_completed_modal = true;
                            state.completion_successed = res;
                        }
                        _ => {}
                    },
                    _ => {}
                },
                PlayerMessage::SliderValueChanged(val) => {
                    state.player.set_slider_value(val);
                }
            },
            GurafuMessage::AlgorithmTick => match state.canvas.step_algorithm() {
                Some(mes) => match mes {
                    CanvasMessage::AlgorithmFinished(res) => {
                        state.show_completed_modal = true;
                        state.completion_successed = res;
                    }
                    _ => {}
                },
                _ => {}
            },
        }
    }

    fn subscription(state: &GurafuApplication) -> Subscription<GurafuMessage> {
        if state.player.playing {
            time::every(state.player.time_delay).map(|_| GurafuMessage::AlgorithmTick)
        } else {
            Subscription::none()
        }
    }

    fn view(state: &GurafuApplication) -> iced::Element<'_, GurafuMessage> {
        if state.show_completed_modal {
            if state.completion_successed {
                text("Выполнение алгоритма завершилось успешно").into()
            } else {
                text("Выполнение алгоритма завершилось с ошибкой").into()
            }
        } else {
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
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}
