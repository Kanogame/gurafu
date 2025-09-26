use iced::{
    Border, Theme, color,
    widget::{
        container,
        svg::{self, Status},
    },
};

pub fn pane_grid_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(iced::Background::Color(palette.background.base.color)),
        border: Border {
            width: 1.0,
            radius: 0.into(),
            color: palette.background.weak.color,
        },
        ..container::Style::default()
    }
}

pub fn button_svg_style(theme: &Theme, _: Status) -> svg::Style {
    let palette = theme.extended_palette();

    svg::Style {
        color: Some(color!(0xffffff)),
    }
}
