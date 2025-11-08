use iced::{
    Font, font,
    widget::{button, column, row, svg, text},
};

use crate::gurafu_app::{styles, svg_button::svg_button};

pub struct FileState {
    // state
}

#[derive(Debug, Clone)]
pub enum FileMessage {
    NewFile,
    OpenFile,
    SaveFile,
}

impl FileState {
    pub fn new() -> Self {
        return FileState {};
    }

    pub fn view(&self) -> iced::Element<'_, FileMessage> {

        column![
            text("Файлы").size(16).font(Font {
                weight: font::Weight::Bold,
                ..Font::default()
            }),
            column![
                svg_button("Новый файл".into(), "assets/icons/add-node.svg".into(), FileMessage::NewFile),
                svg_button("Открыть файл".into(), "assets/icons/open.svg".into(), FileMessage::OpenFile),
                svg_button("Сохранить файл".into(), "assets/icons/save.svg".into(), FileMessage::SaveFile),
            ]
            .spacing(15)
        ]
        .padding(5)
        .spacing(5)
        .into()
    }
}
