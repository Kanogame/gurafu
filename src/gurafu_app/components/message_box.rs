use iced::{
    Alignment, Color, Length::Fill, widget::{button, column, container, row, text}
};

use crate::gurafu_app::styles::message_box_style;

pub struct MessageBoxState {
    pub header_text: String,
    pub message_text: String,
}

#[derive(Debug, Clone)]
pub enum MessageBoxMessage {
    Close,
}

impl MessageBoxState {
    pub fn new() -> Self {
        MessageBoxState {
            header_text: String::new(),
            message_text: String::new(),
        }
    }

    pub fn view(state: &MessageBoxState) -> iced::Element<'_, MessageBoxMessage> {
        container(
            column![
                text(state.header_text.clone()).size(24),
                column![
                    text(state.message_text.clone()).size(16),
                    row![
                        button(text("Закрыть").align_x(Alignment::Center).color(Color::WHITE))
                            .on_press(MessageBoxMessage::Close)
                            .width(Fill),
                    ]
                ]
                .spacing(10)
            ]
            .spacing(20),
        )
        .width(400)
        .padding(10)
        .style(message_box_style)
        .into()
    }
}
