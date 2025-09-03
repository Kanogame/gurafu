use iced::alignment::Horizontal;
use iced::executor;
use iced::widget::{column, container, row, scrollable, text, Button, PaneGrid};
use iced::{Application, Command, Element, Length, Settings, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PaneId {
    Left,
    Editor,
    Bottom,
}

#[derive(Clone, Debug)]
enum AppMessage {
    PaneGrid(iced::pane_grid::Message<PaneId>),
    LeftButton,
    BottomButton,
    // Messages forwarded from pane contents (example)
    FromLeft(LeftMsg),
    FromEditor(EditorMsg),
}

#[derive(Clone, Debug)]
enum LeftMsg {
    NewFile,
}
#[derive(Clone, Debug)]
enum EditorMsg {
    Run,
}

struct App {
    pane_state: iced::pane_grid::State<PaneId>,
}

impl Default for App {
    fn default() -> Self {
        let mut state = iced::pane_grid::State::new();
        // create initial splits:
        // left | editor
        let root = state.split(iced::pane_grid::Node::root(), iced::pane_grid::Direction::Horizontal, &PaneId::Left, &PaneId::Editor, 0.25);
        // editor above bottom
        state.split(root.right(), iced::pane_grid::Direction::Vertical, &PaneId::Editor, &PaneId::Bottom, 0.75);

        Self { pane_state: state }
    }
}

impl Application for App {
    type Message = AppMessage;
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn new(_: ()) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String { "PanePanel example".into() }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMessage::PaneGrid(m) => {
                // forward to pane state so resizing works
                self.pane_state.update(m);
            }
            AppMessage::LeftButton => {
                println!("Left top-level button pressed");
            }
            AppMessage::BottomButton => {
                println!("Bottom top-level button pressed");
            }
            AppMessage::FromLeft(lm) => match lm {
                LeftMsg::NewFile => println!("New file from left component"),
            },
            AppMessage::FromEditor(em) => match em {
                EditorMsg::Run => println!("Run from editor component"),
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        // Build pane grid with a concise view_fn that uses PanePanel components
        let view_fn = |pane: PaneId, _state: &iced::pane_grid::ContentState, _max: bool| {
            match pane {
                PaneId::Left => {
                    // create a PanePanel that wraps a list + a button; map local messages to AppMessage::FromLeft
                    let left_content: Element<LeftMsg> = left_content();
                    PanePanel::new("Explorer", left_content)
                        .with_toolbar(vec![Button::new(text("New")).on_press(LeftMsg::NewFile)])
                        .view()
                        .map(AppMessage::FromLeft)
                }
                PaneId::Editor => {
                    let editor_content: Element<EditorMsg> = editor_content();
                    PanePanel::new("Editor", editor_content)
                        .with_toolbar(vec![Button::new(text("Run")).on_press(EditorMsg::Run)])
                        .view()
                        .map(AppMessage::FromEditor)
                }
                PaneId::Bottom => {
                    let bottom: Element<AppMessage> = bottom_content();
                    // bottom is already in AppMessage space, so wrap via a PanePanel that accepts AppMessage directly
                    PanePanel::new("Terminal", bottom)
                        .with_toolbar(vec![Button::new(text("Clear")).on_press(AppMessage::BottomButton)])
                        .view()
                        .map(|m| m) // identity map (already AppMessage)
                }
            }
        };

        let pane_grid = PaneGrid::new(&self.pane_state, view_fn)
            .on_message(AppMessage::PaneGrid)
            .spacing(4)
            .min_size(80)
            .style(|theme| pane_grid_style(theme));

        container(pane_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(8)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

// -------------------- PanePanel component --------------------
//
// Generic small component that accepts a title, toolbar widgets, and a body (Element<Msg>).
// It exposes a builder-like API: new(title, body).with_toolbar(vec![...]).view() -> Element<Msg>
//
struct PanePanel<Msg> {
    title: String,
    toolbar: Vec<Button<'static, Msg>>,
    body: Element<'static, Msg>,
}

impl<Msg> PanePanel<Msg> {
    pub fn new<T: Into<String>>(title: T, body: Element<'static, Msg>) -> Self {
        Self {
            title: title.into(),
            toolbar: Vec::new(),
            body,
        }
    }

    pub fn with_toolbar(mut self, buttons: Vec<Button<'static, Msg>>) -> Self {
        self.toolbar = buttons;
        self
    }

    pub fn view(self) -> Element<'static, Msg> {
        let title = text(self.title).size(14).horizontal_alignment(Horizontal::Left);
        let toolbar_row = if self.toolbar.is_empty() {
            row![]
        } else {
            row!(self.toolbar).spacing(6)
        };

        let header = row![title, toolbar_row].spacing(8);

        let body_scroll = scrollable(self.body).height(Length::Fill);

        column![header, body_scroll].spacing(8).padding(8).into()
    }
}

// -------------------- Example content factories --------------------

fn left_content() -> Element<'static, LeftMsg> {
    let items = (1..8).fold(column![].spacing(6), |col, i| {
        col.push(container(text(format!("file_{}.rs", i))).padding(6))
    });

    column![text("Project"), Button::new(text("Create")).on_press(LeftMsg::NewFile), scrollable(items)]
        .spacing(8)
        .padding(6)
        .into()
}

fn editor_content() -> Element<'static, EditorMsg> {
    column![
        text("main.rs"),
        container(text(SAMPLE_CODE)).padding(8).width(Length::Fill)
    ]
    .spacing(8)
    .padding(6)
    .into()
}

fn bottom_content() -> Element<'static, AppMessage> {
    column![
        text("Terminal output"),
        scrollable(column![text("Build succeeded")].spacing(4).padding(4))
    ]
    .spacing(8)
    .padding(6)
    .into()
}

// -------------------- Styles & constants --------------------

fn pane_grid_style(_theme: &Theme) -> iced::pane_grid::Style {
    iced::pane_grid::Style {
        background: iced::Color::from_rgb(0.06, 0.06, 0.07),
        border_radius: 6.0,
        split: iced::pane_grid::Split::Line {
            width: 3.0,
            color: iced::Color::from_rgb(0.12, 0.12, 0.13),
        },
        gap: 4.0,
        ..Default::default()
    }
}

const SAMPLE_CODE: &str = r#"fn main() {
    println!("hello");
}"#;

fn main() -> iced::Result {
    App::run(Settings::default())
}
