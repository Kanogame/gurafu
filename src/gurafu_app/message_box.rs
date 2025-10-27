use iced::{
    Alignment,
    Length::Fill,
    widget::{button, canvas::path::lyon_path::geom::euclid::Length, column, container, row, text},
};

use crate::gurafu_app::styles::message_box_style;

pub struct MessageBoxState {
    pub message_text: String,
}

#[derive(Debug, Clone)]
pub enum MessageBoxMessage {
    Close,
}

impl MessageBoxState {
    pub fn new() -> Self {
        MessageBoxState {
            message_text: String::new(),
        }
    }

    pub fn view(state: &MessageBoxState) -> iced::Element<'_, MessageBoxMessage> {
        container(
            column![
                text("Результаты работы алгоритма").size(24),
                column![
                    text(state.message_text.clone()).size(16),
                    row![
                        button(text("Закрыть").align_x(Alignment::Center))
                            .on_press(MessageBoxMessage::Close)
                            .width(Fill),
                    ]
                ]
                .spacing(10)
            ]
            .spacing(20),
        )
        .width(300)
        .padding(10)
        .style(message_box_style)
        .into()
    }
}
