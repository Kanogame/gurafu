
use iced::{
    Color,
    widget::canvas::{self, Path, path::lyon_path::geom::Vector},
};

use crate::gurafu_app::canvas::{camera::{Camera, WorldPoint}, drawable::{DrawablePath, node::NodeSerializable}};

#[derive(Clone, Default)]
pub struct Link {
    start: WorldPoint,
    end: WorldPoint,
    line_width: f32,
    arrowhead_size: f32,
    offset: f32,
    color: Color,
}


impl DrawablePath for Link {
    fn into_path(&self, camera: &Camera) -> Path {
        let mut screen_start = camera.world_to_screen(self.start);
        let mut screen_end = camera.world_to_screen(self.end);

        // line
        // how this works:
        // we have a vector
        let v = Vector::new(screen_end.x - screen_start.x, screen_end.y - screen_start.y);

        // we get a vector of length 1
        let unit = iced::Vector {
            x: v.x / v.length() / camera.scale,
            y: v.y / v.length() / camera.scale,
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

        screen_start = screen_start + unit * self.offset;
        screen_end = screen_end - unit * self.offset;

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

        if v.length() < self.arrowhead_size {
            canvas::Path::new(move |p| {
                p.move_to(tip);
                p.line_to(left_base);
                p.line_to(right_base);
                p.close();
            })
        } else {
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

    fn get_path_color(&self) -> Color {
        return self.color;
    }
}

impl Link {
    pub fn from_world_points(source: WorldPoint, target: WorldPoint) -> Self {
        Self {
            start: source,
            end: target,
            line_width: 5.0,
            arrowhead_size: 10.0,
            offset: 30.0,
            color: Color::from_rgb8(80, 80, 80),
        }
    }

    pub fn from_world_points_new_link(source: WorldPoint, target: WorldPoint) -> Self {
        Self {
            start: source,
            end: target,
            line_width: 10.0,
            arrowhead_size: 30.0,
            offset: 0.0,
            color: Color::from_rgb8(80, 80, 80),
        }
    }

    pub fn from_nodes_serializable(source: &NodeSerializable, target: &NodeSerializable) -> Self {
        Self {
            start: source.into(),
            end: target.into(),
            line_width: 5.0,
            arrowhead_size: 10.0,
            offset: 30.0,
            color: Color::from_rgb8(80, 80, 80),
        }
    }

    pub fn highlight_selected(&mut self) {
        self.color = Color::from_rgb8(0, 255, 255);
    }

    pub fn highlight_path(&mut self) {
        self.color = Color::from_rgb8(144, 238, 144)
    }

    pub fn reset_highlight(&mut self) {
        self.color = Color::from_rgb8(80, 80, 80);
    }
}
