use iced::widget::{Row, button, column, svg, text};

use crate::gurafu_app::styles;

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

    fn name(&self) -> &str {
        match self {
            ToolbarOptions::Hand => "Hand",
            ToolbarOptions::Node => "Node",
            ToolbarOptions::Connection => "Connection",
        }
    }

    fn icon(&self) -> &str {
        match self {
            ToolbarOptions::Hand => "assets/icons/hand.svg",
            ToolbarOptions::Node => "assets/icons/add-node.svg",
            ToolbarOptions::Connection => "assets/icons/add-link.svg",
        }
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

    //pub fn update(state: &mut ToolbarState, message: ToolbarMessage) {}

    pub fn view(_: &ToolbarState) -> iced::Element<'_, ToolbarMessage> {
        column![
            Row::with_children(ToolbarOptions::VALUES.map(|el| {
                button(svg(el.icon()).style(styles::button_svg_style))
                    .on_press(el.to_message())
                    .into()
            })),
            //text(state.state.name()),
        ]
        .into()
    }
}
