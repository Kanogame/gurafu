use iced::widget::{button, column, pane_grid, text};


pub struct GurafuApplication {
    panes: pane_grid::State<Pane>,
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
        let (panes, _) = pane_grid::State::new(Pane::Canvas);

        GurafuApplication {
            panes,
        }
    }


    fn update(&mut self, message: GurafuMessage)  {
        match message {
            GurafuMessage::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
            }

        }
    }
    fn view(&self) -> iced::Element<GurafuMessage> {
        pane_grid(&self.panes, |pane, state, is_maximized| {
            pane_grid::Content::new({
                match state {
                    Pane::Canvas => text("this will be a canvas"),
                    Pane::File => text("this will be a file managment"),
                    Pane::Player => text("this will be a player"),
                }
            })
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