pub mod inference;

use iced::widget::{column, horizontal_space, row, text, vertical_space};
use iced::Element;

#[derive(Debug, Clone)]
pub enum Screen {
    Loading,
    Inference,
}

pub fn loading<'a, Message: 'a>() -> Element<'a, Message> {
    column![
        vertical_space(),
        row![horizontal_space(), text("Loading..."), horizontal_space()],
        vertical_space(),
    ]
    .into()
    // row![horizontal_space(), text("Loading..."), horizontal_space(),].into()
}
