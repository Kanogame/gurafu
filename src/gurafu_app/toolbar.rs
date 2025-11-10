use iced::{
    Font, font,
    widget::{Column, Row, column, text},
};

use crate::gurafu_app::svg_button::svg_button;

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
            text("Граф").size(16).font(Font {
                weight: font::Weight::Bold,
                ..Font::default()
            }),
            text("Выбранный инструмент: ".to_string() + state.state.name()),
            Column::with_children(
                ToolbarOptions::VALUES
                    .map(|el| { svg_button(el.name().into(), el.icon().into(), el.to_message()) })
            )
            .spacing(15)
            .padding([5, 0]),
        ]
        .spacing(5)
        .padding([5, 10])
        .into()
    }
}
