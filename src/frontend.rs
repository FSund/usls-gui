use std::sync::Arc;

use crate::backend;
use crate::backend::{Input, Output};
use crate::io;

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

use futures::{SinkExt, Stream, StreamExt};
// use iced::widget::{button, center, checkbox, column, container, row, text, Space};
// use iced::{Element, Font, Length, Subscription, Task};
// use std::sync::mpsc::{self, Receiver, Sender};
// use tokio::sync::mpsc;
use futures::channel::mpsc;
use image::DynamicImage;
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

#[derive(Default)]
pub struct Example {
    // default: bool,
    // styled: bool,
    // custom: bool,
    image: Option<Arc<DynamicImage>>,

    // Channels for communication
    backend_tx: Option<mpsc::Sender<Input>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    RunDetection(Arc<DynamicImage>),
    LoadImage,
    ImageLoaded(Result<image::DynamicImage, io::LoadError>),
    Backend(Output),
}

impl Example {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                ..Default::default()
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "Hello, World!".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadImage => {
                log::debug!("Load Image button pressed!");

                // Open file dialog to load an image
                return Task::perform(io::open_image(), Message::ImageLoaded);
            }
            Message::ImageLoaded(result) => {
                if let Ok(image) = result {
                    log::info!("Image loaded successfully!");
                    self.image = Some(Arc::new(image));
                } else {
                    log::error!("Failed to load image: {:?}", result);
                }
            }
            Message::RunDetection(image) => {
                // No need to pass the image here
                log::debug!("Button pressed!");

                // Send message to backend
                if let Some(tx) = &self.backend_tx {
                    let msg = Input::ProcessImage(image);

                    // Spawn a new task to send the message
                    let mut tx = tx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tx.send(msg).await {
                            eprintln!("Failed to send message to backend: {:?}", e);
                        }
                    });
                }
            }
            Message::Backend(message) => {
                log::debug!("Received message from backend: {:?}", message);
                match message {
                    Output::Ready(tx) => {
                        log::info!("Backend is ready to receive messages.");
                        self.backend_tx = Some(tx);
                    }
                    Output::DetectionResults(detections) => {
                        log::info!("Received detection results: {:?}", detections);
                    }
                    Output::Error(err) => {
                        log::error!("Received error: {:?}", err);
                    }
                    Output::Progress(progress) => {
                        log::info!("Received progress: {:?}", progress);
                    }
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let image = if let Some(image) = &self.image {
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

        let load_image_button = button("Load Image").on_press(Message::LoadImage);
        let detect_button = if let Some(image) = &self.image {
            button("Run detection").on_press(Message::RunDetection(image.clone()))
        } else {
            button("Run detection")
        };

        let menu = row![load_image_button, detect_button]
            .spacing(20)
            .align_y(iced::alignment::Vertical::Bottom);
        let menu = container(menu).height(50);

        let content = column![
            Space::new(Length::Fill, Length::Fill),
            image,
            menu,
            Space::new(Length::Fill, Length::Fill),
        ]
        .align_x(iced::alignment::Horizontal::Center);

        center(content).into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        // Subscribe to messages from the backend
        //     if let Some(rx) = &self.frontend_rx {
        //         iced::Subscription::run(
        //             let rx_clone = rx.clone();
        //             iced::stream::channel(
        //                 100, // Buffer size
        //                 |mut output| async move {
        //                     while let Some(msg) = rx_clone.next().await {
        //                         match next_msg {
        //                             Some(Output::DetectionResults(detections)) => {
        //                                 log::info!("Received detection results: {:?}", detections);
        //                                 // Return the result and the updated state
        //                                 (Some(Message::ButtonPressed), rx)
        //                             }
        //                             Some(Output::Error(err)) => {
        //                                 log::error!("Received error: {:?}", err);
        //                                 (Some(Message::ButtonPressed), rx)
        //                             }
        //                             Some(Output::Progress(progress)) => {
        //                                 log::info!("Received progress: {:?}", progress);
        //                                 (Some(Message::ButtonPressed), rx)
        //                             }
        //                             None => {
        //                                 // Channel is closed
        //                                 (None, rx)
        //                             }
        //                         }
        //                     }
        //                 }
        //             )
        //         )
        //     } else {
        //         iced::Subscription::none()
        //     }

        // Subscription::run(backend::connect)

        Subscription::run(backend::connect).map(Message::Backend)
    }
}

// fn connect(rx: mpsc::Receiver<Output>) -> impl Stream<Item = Message> {
//     async_stream::stream! {
//         let mut rx = rx;
//         while let Some(output) = rx.next().await {
//             match output {
//                 Output::DetectionResults(_) => yield Message::ButtonPressed,
//                 Output::Error(_) => yield Message::ButtonPressed,
//                 Output::Progress(_) => yield Message::ButtonPressed,
//                 // Add appropriate message mappings based on the outputs
//             }
//         }
//     }
// }

// fn sub(mut rx: mpsc::Receiver<Output>) -> impl Stream<Item = Message> {
//     iced::stream::channel(100, |mut output| async move {
//         while let Some(msg) = rx.next().await {
//             match msg {
//                 Output::DetectionResults(detections) => {
//                     log::info!("Received detection results: {:?}", detections);
//                     // Handle detection results
//                 }
//                 Output::Error(err) => {
//                     log::error!("Received error: {:?}", err);
//                     // Handle error
//                 }
//                 Output::Progress(progress) => {
//                     log::info!("Received progress: {:?}", progress);
//                     // Handle progress
//                 }
//             }
//         }
//     })
// }
