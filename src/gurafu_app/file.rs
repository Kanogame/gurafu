use iced::{
    Font, font,
    widget::{button, column, row, svg, text},
};

pub struct FileState {
    // state
}

#[derive(Debug, Clone)]
pub enum FileMessage {}

impl FileState {
    pub fn new() -> Self {
        return FileState {};
    }

    pub fn view(&self) -> iced::Element<'_, FileMessage> {
        column![
            text("Browser").size(16).font(Font {
                weight: font::Weight::Bold,
                ..Font::default()
            }),
            row![
                button(svg("assets/icons/open.svg")), //.on_press(FileMessage::OpenFile),
                button(svg("assets/icons/save.svg")), //.on_press(FileMessage::SaveFile),
            ]
            .spacing(20)
        ]
        .padding(5)
        .spacing(5)
        .into()
    }
}
