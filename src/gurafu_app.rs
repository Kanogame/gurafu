use std::fmt::Debug;

use iced::Length::{self, Fill};
use iced::time::{self};
use iced::widget::{Space, button, column, container, row, stack, svg, text};
use iced::{Font, Task, font};

use iced::{Settings, Subscription, widget::pane_grid};
use rfd::{AsyncFileDialog, FileHandle};

use crate::gurafu_app::canvas::CanvasSerializable;
use crate::gurafu_app::canvas::algorithms::hierholzer::HierholzerState;
use crate::gurafu_app::canvas::graph_algorithm::AlgorithmMessage;
use crate::gurafu_app::components::message_box::{self, MessageBoxMessage};
use crate::gurafu_app::components::modal::modal;
use crate::gurafu_app::file::FileMessage;
use crate::gurafu_app::{canvas::CanvasMessage, player::PlayerMessage, toolbar::ToolbarMessage};

mod canvas;
mod components;
mod file;
mod player;
mod styles;
mod toolbar;

pub struct GurafuApplication {
    panes: pane_grid::State<Pane>,
    file: file::FileState,
    canvas: canvas::CanvasState<HierholzerState>,
    toolbar: toolbar::ToolbarState,
    player: player::PlayerState,
    message_box: message_box::MessageBoxState,

    modal_content: ModalState,
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
    MessageBox(message_box::MessageBoxMessage),

    FileOpened(Result<String, String>),
    FileSave(FileHandle),
    CloseModal,
    OpenInfo,
    AlgorithmTick,

    Error(String),
}

#[derive(PartialEq)]
enum ModalState {
    Closed,
    Open,
}

pub fn run() -> iced::Result {
    iced::application(
        "Нахождение эйлерова цикла",
        GurafuApplication::update,
        GurafuApplication::view,
    )
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
                ratio: 0.33,
                a: Box::new(pane_grid::Configuration::Pane(Pane::Toolbar)),
                b: Box::new(pane_grid::Configuration::Split {
                    axis: pane_grid::Axis::Horizontal,
                    ratio: 0.66,
                    a: Box::new(pane_grid::Configuration::Pane(Pane::File)),
                    b: Box::new(pane_grid::Configuration::Pane(Pane::Player)),
                }),
            }),
            b: Box::new(pane_grid::Configuration::Pane(Pane::Canvas)),
        });

        GurafuApplication {
            panes,
            file: file::FileState::new(),
            canvas: canvas::CanvasState::new(),
            toolbar: toolbar::ToolbarState::new(),
            player: player::PlayerState::new(),
            message_box: message_box::MessageBoxState::new(),

            modal_content: ModalState::Closed,
        }
    }

    fn update(state: &mut GurafuApplication, message: GurafuMessage) -> Task<GurafuMessage> {
        match message {
            GurafuMessage::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                state.panes.resize(split, ratio);
            }
            GurafuMessage::File(message) => match message {
                FileMessage::OpenFile => {
                    return Task::future(async {
                        let file = AsyncFileDialog::new()
                            .add_filter("json", &["json"])
                            .pick_file()
                            .await;

                        if let Some(file) = file {
                            match std::fs::read_to_string(file.path()) {
                                Ok(content) => GurafuMessage::FileOpened(Ok(content)),
                                Err(e) => GurafuMessage::FileOpened(Err(e.to_string())),
                            }
                        } else {
                            GurafuMessage::FileOpened(Err("Не выбран файл".to_string()))
                        }
                    });
                }
                FileMessage::SaveFile => {
                    return Task::future(async {
                        let file = AsyncFileDialog::new()
                            .add_filter("json", &["json"])
                            .save_file()
                            .await;

                        if let Some(file) = file {
                            GurafuMessage::FileSave(file)
                        } else {
                            GurafuMessage::FileOpened(Err("Не выбран файл".to_string()))
                        }
                    });
                }
                FileMessage::NewFile => {
                    state.toolbar.state = toolbar::ToolbarOption::Hand;
                    state.canvas.set_graph(CanvasSerializable::new().into());
                }
            },
            GurafuMessage::FileOpened(res) => match res {
                Ok(content) => match serde_json::from_str::<CanvasSerializable>(&content) {
                    Ok(graph) => {
                        state.toolbar.state = toolbar::ToolbarOption::Hand;
                        state.canvas.set_graph(graph.into());
                    }
                    Err(er) => {
                        state.open_modal_generic_error(format!(
                            "Неверное форматирование файла графа, ошибка: {}",
                            er
                        ));
                    }
                },
                _ => {}
            },
            GurafuMessage::FileSave(file) => {
                let serializable: CanvasSerializable = state.canvas.clone().into();

                match serde_json::to_string(&serializable) {
                    Ok(json) => {
                        return Task::future(async move {
                            let res = file.write(json.as_bytes()).await;
                            if res.is_err() {
                                return GurafuMessage::Error(format!(
                                    "Не удалось записать файл, ошибка: {:?}",
                                    res.err()
                                ));
                            } else {
                                return GurafuMessage::Error("".to_string());
                            }
                        });
                    }
                    Err(er) => {
                        state.open_modal_generic_error(format!(
                            "Не удалось записать файл, ошибка: {}",
                            er
                        ));
                    }
                }
            }
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
            },
            GurafuMessage::OpenInfo => {
                state.open_modal_about();
            }
            GurafuMessage::Toolbar(message) => match message {
                ToolbarMessage::ChosenOption(new_state) => {
                    state.toolbar.state = new_state.clone();
                    state.canvas.set_toolbar_options(new_state);
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
                    Some(mes) => state.open_modal_algorithm(mes),
                    None => {}
                },
                PlayerMessage::SliderValueChanged(val) => {
                    state.player.set_slider_value(val);
                }
            },
            GurafuMessage::AlgorithmTick => match state.canvas.step_algorithm() {
                Some(mes) => state.open_modal_algorithm(mes),
                None => {}
            },
            GurafuMessage::CloseModal => {
                state.modal_content = ModalState::Closed;
            }
            GurafuMessage::MessageBox(message) => match message {
                MessageBoxMessage::Close => {
                    state.modal_content = ModalState::Closed;
                }
            },
            GurafuMessage::Error(s) => {
                if s.len() > 0 {
                    state.open_modal_generic_error(s);
                }
            }
        }

        Task::none()
    }

    fn subscription(state: &GurafuApplication) -> Subscription<GurafuMessage> {
        if state.player.playing {
            time::every(state.player.time_delay).map(|_| GurafuMessage::AlgorithmTick)
        } else {
            Subscription::none()
        }
    }

    fn view(state: &GurafuApplication) -> iced::Element<'_, GurafuMessage> {
        stack![
            column![
                row![
                    text("Программа для нахождения цикла Эйлера")
                        .size(24)
                        .font(Font {
                            weight: font::Weight::Bold,
                            ..Font::default()
                        })
                        .width(Fill),
                    button(svg("assets/icons/info.svg").style(styles::button_svg_style))
                        .on_press(GurafuMessage::OpenInfo)
                        .width(80),
                ]
                .padding([5, 10]),
                pane_grid(&state.panes, |_, pane_state, _| match pane_state {
                    Pane::File => pane_grid::Content::new({
                        file::FileState::view(&state.file).map(GurafuMessage::File)
                    })
                    .style(styles::pane_grid_style),
                    Pane::Canvas => pane_grid::Content::new({
                        container(
                            container(
                                canvas::CanvasState::view(&state.canvas).map(GurafuMessage::Canvas),
                            )
                            .style(styles::canvas_container_style)
                            .height(Length::Fill)
                            .width(Length::Fill),
                        )
                        .padding(8)
                        .width(Length::Fill)
                        .height(Length::Fill)
                    })
                    .style(styles::pane_grid_canvas_style),
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
            ],
            if state.modal_content == ModalState::Open {
                container(modal(message_box::MessageBoxState::view(&state.message_box)
                        .map(GurafuMessage::MessageBox),
                    GurafuMessage::CloseModal,
                ))
                .width(Length::Fill)
                .height(Length::Fill)
            } else {
                container(Space::new(0, 0))
            }
        ]
        .into()
    }

    fn open_modal(&mut self, header: String, text: String) {
        self.modal_content = ModalState::Open;
        self.player.playing = false;
        self.message_box.header_text = header;

        self.message_box.message_text = text;
    }

    fn open_modal_algorithm(&mut self, mes: AlgorithmMessage) {
        self.open_modal(mes.get_header(), mes.get_text().clone());
    }

    fn open_modal_generic_error(&mut self, error_text: String) {
        self.open_modal("Произошла ошибка".to_string(), error_text);
    }

    fn open_modal_about(&mut self) {
        self.open_modal("О программе".to_string(),          "Программа для наглядой визуализации нахождения цикла Эйлера.\n Выполнил Иванов Александр Евгеньевич, группа 424-3\n Программа построенна на фреймворке iced для Rust".to_string());
    }

    fn theme(&self) -> iced::Theme {
        styles::get_theme()
    }
}
