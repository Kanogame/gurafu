use iced::widget::{Row, button, row, svg};

pub struct PlayerState {
    pub speed: i32,
    pub playing: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum PlayerMessage {
    PlayPause,
    FastForward,
    NextStep,
}

impl PlayerState {
    pub fn new() -> Self {
        PlayerState {
            speed: 1,
            playing: false,
        }
    }

    pub fn view(state: &PlayerState) -> iced::Element<'_, PlayerMessage> {
        row![
            button(if state.playing {
                svg("assets/icons/play.svg")
            } else {
                svg("assets/icons/pause.svg")
            })
            .on_press(PlayerMessage::PlayPause),
            button(svg("assets/icons/fast-forward.svg")).on_press(PlayerMessage::FastForward),
            button(svg("assets/icons/next.svg")).on_press(PlayerMessage::NextStep),
        ]
        .into()
    }
}
