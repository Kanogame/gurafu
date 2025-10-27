use iced::{
    Alignment,
    Length::{Fill, Shrink},
    widget::{Row, button, center, column, container, row, svg, text},
};

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
            ToolbarOptions::Hand => "Рука",
            ToolbarOptions::Node => "Узел",
            ToolbarOptions::Connection => "Связь",
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

    pub fn view(state: &ToolbarState) -> iced::Element<'_, ToolbarMessage> {
        column![
            Row::with_children(ToolbarOptions::VALUES.map(|el| {
                button(center(
                    row![
                        text(el.name().to_string()),
                        svg(el.icon()).style(styles::button_svg_style).width(Shrink)
                    ]
                    .spacing(10),
                ))
                .on_press(el.to_message())
                .into()
            }))
            .spacing(15)
            .padding([5, 10]),
            text("Выбранный инструмент: ".to_string() + state.state.name()),
        ]
        .spacing(10)
        .padding([5, 0])
        .into()
    }
}
