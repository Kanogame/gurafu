use core::time;

use iced::{
    Alignment, Font, font,
    widget::{button, column, row, slider, svg, text},
};

use crate::gurafu_app::styles;

pub struct PlayerState {
    pub playing: bool,
    pub time_delay: time::Duration,

    slider_value: f32,
    slider_text: String,
}

#[derive(Clone, Copy, Debug)]
pub enum PlayerMessage {
    PlayPause,
    Restart,
    NextStep,
    SliderValueChanged(f32),
}

impl PlayerState {
    pub fn new() -> Self {
        PlayerState {
            playing: false,
            time_delay: time::Duration::from_secs(1),
            slider_value: 2.0,
            slider_text: "1".into(),
        }
    }

    pub fn view(state: &PlayerState) -> iced::Element<'_, PlayerMessage> {
        column![
            text("Timeline").size(16).font(Font {
                weight: font::Weight::Bold,
                ..Font::default()
            }),
            slider(
                0.0..=4.0,
                state.slider_value,
                PlayerMessage::SliderValueChanged
            ),
            text(state.slider_text.clone())
                .size(16)
                .font(Font {
                    weight: font::Weight::Bold,
                    ..Font::default()
                })
                .align_y(Alignment::Center),
            row![
                button(if state.playing {
                    svg("assets/icons/play.svg").style(styles::button_svg_style)
                } else {
                    svg("assets/icons/pause.svg").style(styles::button_svg_style)
                })
                .on_press(PlayerMessage::PlayPause),
                button(svg("assets/icons/replay.svg").style(styles::button_svg_style))
                    .on_press(PlayerMessage::Restart),
                button(svg("assets/icons/next.svg").style(styles::button_svg_style))
                    .on_press(PlayerMessage::NextStep),
            ]
        ]
        .padding(5)
        .spacing(5)
        .into()
    }

    pub fn set_slider_value(&mut self, new_slider_value: f32) {
        self.slider_value = new_slider_value;

        match new_slider_value {
            0.0 => {
                self.slider_text = "0.25".to_string();
                self.time_delay = time::Duration::from_secs(4)
            }
            1.0 => {
                self.slider_text = "0.5".to_string();
                self.time_delay = time::Duration::from_secs(2)
            }
            2.0 => {
                self.slider_text = "1".to_string();
                self.time_delay = time::Duration::from_secs(1)
            }
            3.0 => {
                self.slider_text = "2".to_string();
                self.time_delay = time::Duration::from_millis(50)
            }
            4.0 => {
                self.slider_text = "4".to_string();
                self.time_delay = time::Duration::from_millis(25)
            }
            _ => {}
        }
    }
}
