use iced::{
    Font, font,
    widget::{button, column, row, svg, text},
};

use crate::gurafu_app::styles;

pub struct PlayerState {
    //pub speed: i32,
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
            //speed: 1,
            playing: false,
        }
    }

    pub fn view(state: &PlayerState) -> iced::Element<'_, PlayerMessage> {
        column![
            text("Timeline").size(16).font(Font {
                weight: font::Weight::Bold,
                ..Font::default()
            }),
            row![
                button(if state.playing {
                    svg("assets/icons/play.svg").style(styles::button_svg_style)
                } else {
                    svg("assets/icons/pause.svg").style(styles::button_svg_style)
                })
                .on_press(PlayerMessage::PlayPause),
                button(svg("assets/icons/fast-forward.svg").style(styles::button_svg_style))
                    .on_press(PlayerMessage::FastForward),
                button(svg("assets/icons/next.svg").style(styles::button_svg_style))
                    .on_press(PlayerMessage::NextStep),
            ]
        ]
        .padding(5)
        .spacing(5)
        .into()
    }
}
