use std::fmt::Debug;
use std::sync::Arc;

use iced::Length::{self, Fill};
use iced::theme::palette::Extended;
use iced::theme::{Custom, Palette};
use iced::time::{self};
use iced::widget::{button, column, container, row, svg, text};
use iced::{Color, Font, Task, font};

use iced::{Settings, Subscription, widget::pane_grid};
use rfd::{AsyncFileDialog, FileHandle};
use serde::{Deserialize, Serialize};

use crate::gurafu_app::canvas::{AlgorithmMessage, CanvasSerializable};
use crate::gurafu_app::file::FileMessage;
use crate::gurafu_app::message_box::MessageBoxMessage;
use crate::gurafu_app::{canvas::CanvasMessage, player::PlayerMessage, toolbar::ToolbarMessage};

mod canvas;
mod file;
mod message_box;
mod modal;
mod player;
mod styles;
mod toolbar;
mod svg_button;

pub struct GurafuApplication {
    panes: pane_grid::State<Pane>,
    file: file::FileState,
    canvas: canvas::CanvasState,
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
                },
                FileMessage::NewFile => {
                    state.load_graph(CanvasSerializable::new());
                }
            },
            GurafuMessage::FileOpened(res) => match res {
                Ok(content) => {
                    match serde_json::from_str::<CanvasSerializable>(&content) {
                        Ok(graph) => {
                            state.load_graph(graph);
                        }
                        Err(er) => {
                            state.open_modal_generic_error(format!("Неверное форматирование файла графа, ошибка: {}", er));
                        }
                    }
                }
                _ => {}
            },
            GurafuMessage::FileSave(file) => {
                let serializable: CanvasSerializable = state.canvas.clone().into();

                match serde_json::to_string(&serializable) {
                    Ok(json) => {
                        return Task::future(async move {
                            let res = file.write(json.as_bytes()).await;
                            if res.is_err() {
                                return GurafuMessage::Error(format!("Не удалось записать файл, ошибка: {:?}", res.err()));
                            } else {
                                return GurafuMessage::Error("".to_string());
                            }
                        });
                    }
                    Err(er) => {
                        state.open_modal_generic_error(format!("Не удалось записать файл, ошибка: {}", er));
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
                        AlgorithmMessage::AlgorithmSuccess(res) => {
                            state.open_modal_algo_success(res);
                        }
                        AlgorithmMessage::AlgorithmFail => {
                            state.open_modal_algo_fail();
                        }
                    },
                    None => {}
                },
                PlayerMessage::SliderValueChanged(val) => {
                    state.player.set_slider_value(val);
                }
            },
            GurafuMessage::AlgorithmTick => match state.canvas.step_algorithm() {
                Some(mes) => match mes {
                    AlgorithmMessage::AlgorithmSuccess(res) => {
                        state.open_modal_algo_success(res);
                    },
                    AlgorithmMessage::AlgorithmFail => {
                            state.open_modal_algo_fail();
                        },
                },
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
        let layout = column![
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
                            canvas::CanvasState::view(&state.canvas).map(GurafuMessage::Canvas))
                        .style(styles::pane_grid_style)
                        .center(Length::Fill)
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                    .style(styles::canvas_container_style)
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
        ];

        match state.modal_content {
            ModalState::Open => modal::modal(
                layout,
                message_box::MessageBoxState::view(&state.message_box)
                    .map(GurafuMessage::MessageBox),
                GurafuMessage::CloseModal,
            )
            .into(),
            ModalState::Closed => layout.into(),
        }
    }

    fn open_modal_algo_fail(&mut self) {
        self.modal_content = ModalState::Open;
        self.player.playing = false;
        self.message_box.header_text = "Результаты работы алгоритма".to_string();
 
        self.message_box.message_text =
            "Алгоритм завершился, Эйлеров цикл не найден".to_string();
    }

    fn open_modal_algo_success(&mut self, circuit: Vec<usize>) {
        self.modal_content = ModalState::Open;
        self.player.playing = false;
        self.message_box.header_text = "Результаты работы алгоритма".to_string();
        self.message_box.message_text = format!("Алгоритм выполнен успешно, Эйлеров цикл: {}", 
            circuit
                .iter()
                .map(|el| el.to_string())
                .collect::<Vec<String>>()
                .join(" -> ")
        );

        self.canvas.reset_algorithm();
       
    }

    fn open_modal_generic_error(&mut self, error_text: String) {
        self.modal_content = ModalState::Open;
        self.player.playing = false;

        self.message_box.header_text = "Произошла ошибка".to_string();
        self.message_box.message_text =error_text;
    }

    fn open_modal_about(&mut self) {
        self.modal_content = ModalState::Open;
        self.player.playing = false;

        self.message_box.header_text = "О программе".to_string();
        self.message_box.message_text =
            "Программа для наглядой визуализации нахождения цикла Эйлера.\n Выполнил Иванов Александр Евгеньевич, группа 424-3\n Программа построенна на фреймворке iced для Rust".to_string();
    }

    fn load_graph(&mut self, graph: CanvasSerializable) {
        self.canvas.graph = graph.into();
        self.canvas.reset_algorithm();
    }

    fn theme(&self) -> iced::Theme {
        let c = Custom::new("Gurafu_theme".to_string(), Palette {
                    primary: Color::from_rgb8(74, 144, 216),
                    background: Color::from_rgb8(232, 232, 232),
                    ..iced::Theme::Light.palette()
                });

        iced::Theme::Custom(Arc::new(
            c
        ))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    x: f32,
    y: f32,
}
