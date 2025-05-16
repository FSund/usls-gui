// use crate::backend::{Input, Output};
use crate::backend;
use crate::frontend::{Message, ZeroShotRust};
// use crate::io;

// use std::sync::Arc;
// use iced::border;
// use iced::keyboard;
use iced::widget::canvas::{Path, Stroke};
use iced::widget::{
    button, canvas, center, checkbox, column, container, horizontal_space, pick_list, row,
    scrollable, text, Space,
};
use iced::{mouse, Vector};
// use iced::Size;
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

// pub const DEFAULT_IMAGE: &[u8] = include_bytes!("../../assets/bus.jpg");
// pub const DEFAULT_IMAGE: &[u8] =
//     include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/bus.jpg"));

fn square<'a>(
    width: impl Into<Length> + Copy,
    height: impl Into<Length> + Copy,
) -> Element<'a, Message> {
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

            // debugging //
            // let rect = Rectangle::with_size((480, 640).into());
            let rect = Path::rectangle((0.0, 0.0).into(), (480.0, 640.0).into());
            // let color = iced::Color::from_rgb8(255, 0, 0);
            // let style = iced::widget::canvas::Style::Solid(color);
            // let fill = iced::widget::canvas::Fill::default();
            let fill: canvas::Fill = iced::Color::from_rgb8(255, 0, 0).into();
            frame.fill(&rect, fill);

            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                // palette.background.strong.color,
                iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5),
            );

            frame.stroke_rectangle(
                Point::ORIGIN,
                (10.0, 10.0).into(),
                Stroke {
                    ..Stroke::default()
                },
            );

            let center = frame.center();
            frame.translate(Vector::new(center.x, center.y));

            let r = Rectangle::with_radius(72.0);
            frame.fill_rectangle(
                (r.x, r.y).into(),
                r.size(),
                iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5),
            );

            // let im = iced::widget::image::Handle::from_bytes(DEFAULT_IMAGE.to_vec());
            // frame.draw_image(Rectangle::with_radius(480.0), &im);

            vec![frame.into_geometry()]
        }
    }

    canvas(Square).width(width).height(height).into()
}

#[derive(Debug, Clone)]
pub struct Image {
    image: Option<iced::advanced::image::Handle>,
    width: u32,
    height: u32,
}

impl Image {
    pub fn new(image: &image::DynamicImage) -> Self {
        let rgba = image.to_rgba8(); // copies image!
        let handle = iced::advanced::image::Handle::from_rgba(
            rgba.width(),
            rgba.height(),
            rgba.as_raw().to_vec(),
        );
        Self {
            image: Some(handle),
            width: rgba.width(),
            height: rgba.height(),
        }
    }
}

impl Default for Image {
    fn default() -> Self {
        Self {
            image: None,
            width: 640,
            height: 480,
        }
    }
}

impl canvas::Program<Message> for Image {
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

        // debugging //
        // let rect = Rectangle::with_size((480, 640).into());
        let rect = Path::rectangle((0.0, 0.0).into(), (480.0, 640.0).into());
        // let color = iced::Color::from_rgb8(255, 0, 0);
        // let style = iced::widget::canvas::Style::Solid(color);
        // let fill = iced::widget::canvas::Fill::default();
        let fill: canvas::Fill = iced::Color::from_rgb8(255, 0, 0).into();
        frame.fill(&rect, fill);

        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            // palette.background.strong.color,
            iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5),
        );

        frame.stroke_rectangle(
            Point::ORIGIN,
            (10.0, 10.0).into(),
            Stroke {
                ..Stroke::default()
            },
        );

        let center = frame.center();
        frame.translate(Vector::new(center.x, center.y));

        let r = Rectangle::with_radius(72.0);
        frame.fill_rectangle(
            (r.x, r.y).into(),
            r.size(),
            iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5),
        );

        // let im = iced::widget::image::Handle::from_bytes(DEFAULT_IMAGE.to_vec());
        // frame.draw_image(Rectangle::with_radius(480.0), &im);

        if let Some(image) = &self.image {
            // let bounds = Rectangle::with_radius(200.0);
            let size: iced::Size<f32> = (self.width as f32, self.height as f32).into();
            let bounds = Rectangle::with_size(size);
            frame.draw_image(bounds, image);
        }

        vec![frame.into_geometry()]
    }
}

pub fn view(app: &ZeroShotRust) -> Element<Message> {
    let image: Element<Message> = if let Some(image) = &app.image {
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
        // square(640, 480)

        canvas(&app.inference_state.image)
            .width(640)
            .height(480)
            .into()
    };
    let image = container(image)
        .padding(10)
        .width(640)
        .height(480)
        // .max_width(480)
        // .max_height(640)
        // .width(Length::Fixed(640.0))
        // .height(Length::Fixed(480.0))
        .align_y(iced::alignment::Vertical::Center)
        .align_x(iced::alignment::Horizontal::Center);

    let mut load_image_button = button("Load Image");
    if !app.inference_state.selecting_image {
        load_image_button = load_image_button.on_press(Message::LoadImage);
    };

    let mut detect_button = button("Run detection");
    if let Some(image) = &app.image {
        if app.backend_tx.is_some()
            && app.inference_state.selected_model.is_some()
            && !app.inference_state.busy
        {
            detect_button = detect_button.on_press(Message::Detect(image.clone()));
        };
    };

    log::debug!(
        "backend_tx: {}, selected_model: {}, busy: {}",
        app.backend_tx.is_some(),
        app.inference_state.selected_model.is_some(),
        app.inference_state.busy
    );

    let models = vec![backend::ModelType::Mock, backend::ModelType::GroundingDINO];
    let model_list = pick_list(
        models,
        app.inference_state.selected_model.clone(),
        Message::SelectModel,
    )
    .placeholder("Select a model");

    let menu = row![load_image_button, detect_button]
        .spacing(20)
        .align_y(iced::alignment::Vertical::Bottom);
    let menu = container(menu).height(50);

    let content = column![
        Space::new(Length::Fill, Length::Fill),
        image,
        menu,
        model_list,
        Space::new(Length::Fill, Length::Fill),
    ]
    .align_x(iced::alignment::Horizontal::Center);

    center(content).into()
}

#[derive(Debug, Clone)]
pub struct InferenceState {
    pub selecting_image: bool,
    pub selected_model: Option<backend::ModelType>,
    pub busy: bool,
    // pub detections: Vec<backend::Detection>,
    // pub image: Option<iced::advanced::image::Handle>,
    pub image: Image,
}

impl Default for InferenceState {
    fn default() -> Self {
        Self {
            selecting_image: false,
            selected_model: None,
            busy: false,
            // detections: vec![],
            image: Image::default(),
        }
    }
}
