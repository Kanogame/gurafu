use iced::{
    Element,
    Length::Shrink,
    widget::{button, center, row, svg, text},
};

use crate::gurafu_app::styles;

pub fn svg_button<'a, Message>(
    button_text: String,
    svg_path: String,
    message: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    return button(center(
        row![
            svg(svg_path).style(styles::button_svg_style).width(Shrink),
            text(button_text)
        ]
        .spacing(10),
    ))
    .on_press(message)
    .into();
}
