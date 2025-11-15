use std::sync::Arc;

use iced::{
    Border, Color, Shadow, Theme, Vector, color, theme::{Custom, Palette}, widget::{
        container,
        svg::{self, Status},
    }
};

pub fn pane_grid_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(248, 248, 248))),
        border: Border {
            width: 1.0,
            radius: 0.into(),
            color: palette.background.weak.color,
        },
        ..container::Style::default()
    }
}

pub fn pane_grid_canvas_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(232, 232, 232))),
        border: Border {
            width: 1.0,
            radius: 0.into(),
            color: palette.background.weak.color,
        },
        ..container::Style::default()
    }
}

pub fn canvas_container_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(248, 248, 248))),
        border: Border {
            width: 1.0,
            radius: 8.0.into(),
            color: palette.background.weak.color,
        },
        shadow: Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.25),
            offset: Vector { x: 0.0, y: 0.0 },
            blur_radius: 20.0,
        },
        ..container::Style::default()
    }
}

pub fn message_box_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(iced::Background::Color(palette.background.base.color)),
        border: Border {
            width: 1.0,
            radius: 10.into(),
            color: palette.background.base.color,
        },
        ..container::Style::default()
    }
}

pub fn button_svg_style(_: &Theme, _: Status) -> svg::Style {
    svg::Style {
        color: Some(color!(0xffffff)),
    }
}

pub fn get_theme() -> Theme {
    let c = Custom::new(
            "Gurafu_theme".to_string(),
            Palette {
                primary: Color::from_rgb8(74, 144, 216),
                background: Color::from_rgb8(232, 232, 232),
                ..iced::Theme::Light.palette()
            },
        );

        iced::Theme::Custom(Arc::new(c))
}