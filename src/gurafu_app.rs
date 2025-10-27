use iced::Length::Fill;
use iced::time::{self};
use iced::widget::{button, column, row, svg, text};
use iced::{Font, font};

use iced::{Settings, Subscription, widget::pane_grid};

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

    //FileOpened(Result<C, String>),
    CloseModal,
    OpenInfo,
    AlgorithmTick,
}

enum ModalState {
    Closed,
    AlgorithmEnded,
    About,
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
                ratio: 0.8,
                a: Box::new(pane_grid::Configuration::Pane(Pane::Player)),
                b: Box::new(pane_grid::Configuration::Pane(Pane::File)),
            }),
            b: Box::new(pane_grid::Configuration::Split {
                axis: pane_grid::Axis::Horizontal,
                ratio: 0.1,
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
            message_box: message_box::MessageBoxState::new(),

            modal_content: ModalState::Closed,
        }
    }

    fn update(state: &mut GurafuApplication, message: GurafuMessage) {
        match message {
            GurafuMessage::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                state.panes.resize(split, ratio);
            }
            GurafuMessage::File(message) => match message {
                FileMessage::OpenFile => {
                    //    let dialog = rdf::AsyncFileDialog::new()
                    //        .add_filter("json", &["json"])
                    //        .pick_file();
                    //    Task::future(dialog)
                    //        .and_then(|file| {
                    //        Task::future(move |mut sender| {
                    //        let content = std::fs::read_to_string(file.path())
                    //            .unwrap_or_default();
                    //        let result = serde_json::from_str::<CanvasSerializaion>(&content)
                    //            .map_err(|e| e.to_string());
                    //        let _ = sender.try_send(result);
                    //    })
                    //})
                    //.map(GurafuMessage::FileOpened)
                }
                FileMessage::SaveFile => {}
            },
            //GurafuMessage::FileOpened(_) => {}
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
                        CanvasMessage::AlgorithmFinished(res) => {
                            state.open_modal_after_algo(res);
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
                        state.open_modal_after_algo(res);
                    }
                    _ => {}
                },
                _ => {}
            },
            GurafuMessage::CloseModal => {
                state.modal_content = ModalState::Closed;
            }
            GurafuMessage::MessageBox(message) => match message {
                MessageBoxMessage::Close => {
                    state.modal_content = ModalState::Closed;
                }
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
        ];

        match state.modal_content {
            ModalState::About => modal::modal(
                layout,
                message_box::MessageBoxState::view(&state.message_box)
                    .map(GurafuMessage::MessageBox),
                GurafuMessage::CloseModal,
            )
            .into(),
            ModalState::AlgorithmEnded => modal::modal(
                layout,
                message_box::MessageBoxState::view(&state.message_box)
                    .map(GurafuMessage::MessageBox),
                GurafuMessage::CloseModal,
            )
            .into(),
            ModalState::Closed => layout.into(),
        }
    }

    fn open_modal_after_algo(&mut self, circuit_found: bool) {
        self.modal_content = ModalState::AlgorithmEnded;
        self.player.playing = false;
        self.message_box.header_text = "Результаты работы алгоритма".to_string();
        if circuit_found {
            self.message_box.message_text =
                "Алгоритм выполнен успешно, Эйлеров цикл найден".to_string();
        } else {
            self.message_box.message_text =
                "Алгоритм завершился, Эйлеров цикл не найден".to_string();
        }
    }

    fn open_modal_about(&mut self) {
        self.modal_content = ModalState::About;
        self.player.playing = false;

        self.message_box.header_text = "О программе".to_string();
        self.message_box.message_text =
            "Программа для наглядой визуализации нахождения цикла Эйлера.\n Выполнил Иванов Александр Евгеньевич, группа 424-3\n Программа построенна на фреймворке iced для Rust".to_string();
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Light
    }
}
