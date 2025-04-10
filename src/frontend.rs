use crate::backend::Input;
use crate::io;
use crate::screen::Screen;
use crate::{backend, screen};

use futures::{SinkExt, Stream, StreamExt};
use iced::{Element, Subscription, Task};
use image::DynamicImage;
use std::sync::Arc;

// #[derive(Default)]
pub struct ZeroShotRust {
    screen: Screen,
    pub image: Option<Arc<DynamicImage>>,
    backend_tx: Option<futures::channel::mpsc::Sender<backend::Input>>,

    pub inference_state: screen::inference::InferenceState,
}

#[derive(Debug, Clone)]
pub enum Message {
    Detect(Arc<DynamicImage>),
    LoadImage,
    ImageLoaded(Result<image::DynamicImage, io::LoadError>),
    Backend(backend::Output),
}

impl Default for ZeroShotRust {
    fn default() -> Self {
        Self {
            screen: Screen::Loading,
            image: None,
            backend_tx: None,
            inference_state: screen::inference::InferenceState::default(),
        }
    }
}

impl ZeroShotRust {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                ..Default::default()
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "ZeroShotRust".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Backend(output) => match output {
                backend::Output::Ready(tx) => {
                    log::info!("Backend is ready!");
                    self.backend_tx = Some(tx.clone());
                    self.screen = Screen::Inference;
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::Detect(image) => {
                // No need to pass the image here
                log::debug!("Button pressed!");

                // Send message to backend

                let msg = Input::ProcessImage(image);

                // Spawn a new task to send the message
                if let Some(tx) = &self.backend_tx {
                    let mut tx = tx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tx.send(msg).await {
                            eprintln!("Failed to send message to backend: {:?}", e);
                        }
                    });
                }

                Task::none()
            }
            Message::LoadImage => {
                log::debug!("Load Image button pressed!");

                // Open file dialog to load an image
                self.inference_state.selecting_image = true;
                Task::perform(io::open_image(), Message::ImageLoaded)
            }
            Message::ImageLoaded(result) => {
                self.inference_state.selecting_image = false;
                if let Ok(image) = result {
                    log::info!("Image loaded successfully!");
                    self.image = Some(Arc::new(image));
                } else {
                    log::error!("Failed to load image: {:?}", result);
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::Loading => screen::loading(),
            Screen::Inference => screen::inference::view(self),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        // Always run backend
        let backend = Subscription::run(backend::connect).map(Message::Backend);

        Subscription::batch([backend])
    }
}
