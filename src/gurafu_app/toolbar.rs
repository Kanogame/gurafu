use iced::widget::{Row, button, column, text};

pub struct ToolbarState {
    pub state: ToolbarOptions,
}

#[derive(Debug, Clone)]
pub enum ToolbarOptions {
    Hand,
    Node,
    Connection,
}

#[derive(Debug, Clone)]
pub enum ToolbarMessage {
    ChosenState(ToolbarOptions),
}

impl ToolbarOptions {
    const VALUES: [Self; 3] = [
        ToolbarOptions::Hand,
        ToolbarOptions::Node,
        ToolbarOptions::Connection,
    ];

    pub fn new() -> Self {
        ToolbarOptions::Hand
    }

    fn name(&self) -> String {
        match self {
            ToolbarOptions::Hand => "Hand",
            ToolbarOptions::Node => "Node",
            ToolbarOptions::Connection => "Connection",
        }
        .into()
    }

    fn to_message(&self) -> ToolbarMessage {
        ToolbarMessage::ChosenState(self.clone())
    }
}

impl ToolbarState {
    pub fn new() -> Self {
        ToolbarState {
            state: ToolbarOptions::new(),
        }
    }

    pub fn update(state: &mut ToolbarState, message: ToolbarMessage) {}

    pub fn view(state: &ToolbarState) -> iced::Element<'_, ToolbarMessage> {
        column![
            Row::with_children(
                ToolbarOptions::VALUES
                    .map(|el| button(text(el.name())).on_press(el.to_message()).into())
            ),
            text(state.state.name()),
        ]
        .into()
    }
}
