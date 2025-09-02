use iced::widget::{button, text};

fn main() -> iced::Result {
    iced::run("My App", MyApp::update, MyApp::view)
}

#[derive(Debug, Clone)]
enum Message {
    Increment
}

#[derive(Default)]
struct MyApp {
    counter: u64,
}

impl MyApp {
    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => self.counter += 1,
        }
    }

    fn view(&self) -> iced::Element<Message> {
        button(text(format!("hello world {} times", self.counter))).on_press(Message::Increment).into()
    }
}
