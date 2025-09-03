use iced::widget::{button, column, text};



#[derive(Debug, Default)]
pub struct GurafuApplication {
    // state would go here
}


#[derive(Debug, Clone)]
enum GurafuMessage {
    
}

pub fn run() -> iced::Result {
    iced::application("Gurafu", GurafuApplication::update, GurafuApplication::view)
    .theme(GurafuApplication::theme)
    .run()
}

impl GurafuApplication {
    fn update(&mut self, message: GurafuMessage)  {}
    fn view(&self) -> iced::Element<GurafuMessage> {
        column![
            text("Hello, world!"),
            text("Hello, world!"),
            text("Hello, world!"),
            text("Hello, world!"),
            text("Hello, world!"),
        ]
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
// graph