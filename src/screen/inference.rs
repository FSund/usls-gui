// use crate::backend::{Input, Output};
use crate::frontend::{Message, ZeroShotRust};
// use crate::io;

// use std::sync::Arc;
// use iced::border;
// use iced::keyboard;
use iced::mouse;
use iced::widget::{
    button, canvas, center, checkbox, column, container, horizontal_space, pick_list, row,
    scrollable, text, Space,
};
use iced::{
    color, Center, Element, Fill, Font, Length, Point, Rectangle, Renderer, Subscription, Task,
    Theme,
};
use tokio::time::error::Elapsed;

// use futures::{SinkExt, Stream, StreamExt};
// use iced::widget::{button, center, checkbox, column, container, row, text, Space};
// use iced::{Element, Font, Length, Subscription, Task};
// use std::sync::mpsc::{self, Receiver, Sender};
// use tokio::sync::mpsc;
// use futures::channel::mpsc;
// use image::DynamicImage;
// use image::DynamicImage;
// use std::sync::{Arc, Mutex};
// use std::thread;
// use tracing::instrument::WithSubscriber;

// pub const DEFAULT_IMAGE: &[u8] = include_bytes!("../assets/bus.jpg");

fn square<'a>(size: impl Into<Length> + Copy) -> Element<'a, Message> {
    struct Square;

    impl canvas::Program<Message> for Square {
        type State = ();

        fn draw(
            &self,
            _state: &Self::State,
            renderer: &Renderer,
            theme: &Theme,
            bounds: Rectangle,
            _cursor: mouse::Cursor,
        ) -> Vec<canvas::Geometry> {
            let mut frame = canvas::Frame::new(renderer, bounds.size());

            let palette = theme.extended_palette();

            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                palette.background.strong.color,
            );

            vec![frame.into_geometry()]
        }
    }

    canvas(Square).width(size).height(size).into()
}

pub fn view(app: &ZeroShotRust) -> Element<Message> {
    let image = if let Some(image) = &app.image {
        // Convert DynamicImage to iced image
        let rgba = image.to_rgba8();
        let im = iced::advanced::image::Handle::from_rgba(
            rgba.width(),
            rgba.height(),
            rgba.as_raw().to_vec(),
        );
        iced::widget::image(im).opacity(0.5).into()
    } else {
        // let im = iced::widget::image::Handle::from_bytes(DEFAULT_IMAGE.to_vec());
        // iced::widget::image(im)
        square(480)
    };
    let image = container(image)
        .padding(10)
        .max_width(480)
        .max_height(640)
        .align_y(iced::alignment::Vertical::Center)
        .align_x(iced::alignment::Horizontal::Center);

    let mut load_image_button = button("Load Image");
    if !app.inference_state.selecting_image {
        load_image_button = load_image_button.on_press(Message::LoadImage);
    };

    let mut detect_button = button("Run detection");
    if let Some(image) = &app.image {
        if app.backend_tx.is_some() {
            detect_button = detect_button.on_press(Message::Detect(image.clone()));
        };
    };

    let model_description = if let Some(model_description) = &app.inference_state.model_description
    {
        text(format!("{} is ready!", model_description))
    } else {
        text("Initializing model...")
    };

    let menu = row![load_image_button, detect_button]
        .spacing(20)
        .align_y(iced::alignment::Vertical::Bottom);
    let menu = container(menu).height(50);

    let content = column![
        Space::new(Length::Fill, Length::Fill),
        image,
        menu,
        model_description,
        Space::new(Length::Fill, Length::Fill),
    ]
    .align_x(iced::alignment::Horizontal::Center);

    center(content).into()
}

#[derive(Debug, Clone)]
pub struct InferenceState {
    pub selecting_image: bool,
    pub model_description: Option<String>,
}

impl Default for InferenceState {
    fn default() -> Self {
        Self {
            selecting_image: false,
            model_description: None,
        }
    }
}
