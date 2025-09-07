use iced::widget::{button, column, text};

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
            button(text("open file")), //.on_press(FileMessage::OpenFile),
            button(text("save file")), //.on_press(FileMessage::SaveFile),
        ]
        .into()
    }
}
