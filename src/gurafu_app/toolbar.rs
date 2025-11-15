use iced::{
    Font, font,
    widget::{Column, column, text},
};

use crate::gurafu_app::components::svg_button::svg_button;


pub struct ToolbarState {
    pub state: ToolbarOption,
}

#[derive(Debug, Clone)]
pub enum ToolbarOption {
    Hand,
    Node,
    Connection,
}

#[derive(Debug, Clone)]
pub enum ToolbarMessage {
    ChosenOption(ToolbarOption),
}

impl ToolbarOption {
    const VALUES: [Self; 3] = [
        ToolbarOption::Hand,
        ToolbarOption::Node,
        ToolbarOption::Connection,
    ];

    pub fn new() -> Self {
        ToolbarOption::Hand
    }

    fn name(&self) -> &str {
        match self {
            ToolbarOption::Hand => "Рука",
            ToolbarOption::Node => "Узел",
            ToolbarOption::Connection => "Связь",
        }
    }

    fn icon(&self) -> &str {
        match self {
            ToolbarOption::Hand => "assets/icons/hand.svg",
            ToolbarOption::Node => "assets/icons/add-node.svg",
            ToolbarOption::Connection => "assets/icons/add-link.svg",
        }
    }

    fn to_message(&self) -> ToolbarMessage {
        ToolbarMessage::ChosenOption(self.clone())
    }
}

impl ToolbarState {
    pub fn new() -> Self {
        ToolbarState {
            state: ToolbarOption::new(),
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
                ToolbarOption::VALUES
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
