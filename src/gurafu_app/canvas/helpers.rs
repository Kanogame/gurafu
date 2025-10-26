use iced::widget::canvas;

use crate::gurafu_app::canvas::CanvasMessage;

pub const IGNORED: (canvas::event::Status, Option<CanvasMessage>) =
    (canvas::event::Status::Ignored, None::<CanvasMessage>);
pub const CAPTURED: (canvas::event::Status, Option<CanvasMessage>) =
    (canvas::event::Status::Captured, None::<CanvasMessage>);

pub fn capured_message(message: CanvasMessage) -> (canvas::event::Status, Option<CanvasMessage>) {
    (canvas::event::Status::Captured, Some(message))
}
