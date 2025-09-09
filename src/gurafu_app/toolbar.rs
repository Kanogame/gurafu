use iced::widget::{button, row, text, Row};

pub struct ToolbarState {
    state: ToolbarOptions
}

#[derive(Clone)]
pub enum ToolbarOptions {
    Hand,
    Node,
    Connection
}

pub enum ToolbarMessage {
    ChosenState(ToolbarOptions),
}

impl ToolbarOptions {
    const values: [Self; 3] = [ToolbarOptions::Hand, ToolbarOptions::Node, ToolbarOptions::Connection];

    fn name(&self) -> String {
        match self {
            ToolbarOptions::Hand => "Hand",
            ToolbarOptions::Node => "Node",
            ToolbarOptions::Connection => "Connection"
        }.into()
    }
}

impl ToolbarState {
    pub fn new() -> Self {
        ToolbarState {
            state: ToolbarOptions::Hand,
        }
    }

    pub fn view(&self) -> iced::Element<'_, ToolbarOptions> {
        Row::with_children(ToolbarOptions::values.map(
            |el| 
            button(text(el.name()))
                .on_press(el.clone())
                .into()
            )
        ).into()
    }
}