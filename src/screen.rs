pub mod inference;

use iced::widget::{horizontal_space, row, text};
use iced::Element;

pub enum Screen {
    Loading,
    Inference,
}

pub fn loading<'a, Message: 'a>() -> Element<'a, Message> {
    row![horizontal_space(), text("Loading..."), horizontal_space(),].into()
}
