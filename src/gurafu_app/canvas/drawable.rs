use iced::{
    Point,
    widget::canvas::{
        self, Path,
        path::{
            Builder,
            lyon_path::geom::{Vector, euclid::Vector2D},
        },
    },
};

use crate::gurafu_app::canvas::camera::Camera;

pub trait Drawable {
    fn into_path(&self, camera: &Camera) -> Path;
}

pub struct Circle {
    pub center: Point,
    pub radius: f32,
}

impl Drawable for Circle {
    fn into_path(&self, camera: &Camera) -> Path {
        Path::circle(
            camera.world_to_screen(self.center),
            self.radius / camera.scale,
        )
    }
}

pub struct Arrow {
    pub start: Point,
    pub end: Point,
    pub line_width: f32,
    pub arrowhead_size: f32,
    // ?
}

impl Drawable for Arrow {
    fn into_path(&self, camera: &Camera) -> Path {
        let screen_start = camera.world_to_screen(self.start);
        let screen_end = camera.world_to_screen(self.end);

        // line
        // how this works:
        // we have a vector
        let v = Vector::new(screen_end.x - screen_start.x, screen_end.y - screen_start.y);

        // we get a vector of length 1
        let unit = iced::Vector {
            x: v.x / v.length(),
            y: v.y / v.length(),
        };

        // we get a perpendicular vector to unit
        let perpendicular = iced::Vector::new(-unit.y, unit.x);

        // we multiple perp. vector by half of width
        let half_width = self.line_width / 2.0;
        let offset = perpendicular * half_width;

        let offset_iced = iced::Vector {
            x: offset.x,
            y: offset.y,
        };

        let rectangle_end = screen_end - unit * self.arrowhead_size;

        let points = [
            screen_start + offset_iced,  // top left
            screen_start - offset_iced,  // bottom left
            rectangle_end - offset_iced, // bottom right
            rectangle_end + offset_iced, // top right
        ];

        // Arrowhead
        let tip = screen_end;
        let base_center = screen_end
            - iced::Vector {
                x: unit.x,
                y: unit.y,
            } * self.arrowhead_size;
        let base_width = self.arrowhead_size * 0.5;
        let base_offset = perpendicular * base_width;

        let left_base = base_center + base_offset;
        let right_base = base_center - base_offset;

        canvas::Path::new(move |p| {
            p.move_to(points[0]);
            p.line_to(points[1]);
            p.line_to(points[2]);
            p.line_to(points[3]);
            p.close();

            p.move_to(tip);
            p.line_to(left_base);
            p.line_to(right_base);
            p.close();
        })
    }
}
