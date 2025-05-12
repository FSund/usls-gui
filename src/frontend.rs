use crate::backend::Input;
use crate::io;
use crate::screen::{inference, Screen};
use crate::{backend, screen};

use futures::{SinkExt, Stream, StreamExt};
use iced::{Element, Subscription, Task};
use image::DynamicImage;
use std::sync::Arc;

pub const DEFAULT_IMAGE: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/bus.jpg"));

// #[derive(Default)]
pub struct ZeroShotRust {
    screen: Screen,
    pub image: Option<Arc<DynamicImage>>,
    pub backend_tx: Option<futures::channel::mpsc::Sender<backend::Input>>,
    pub inference_state: screen::inference::InferenceState,
}

#[derive(Debug, Clone)]
pub enum Message {
    Detect(Arc<DynamicImage>),
    LoadImage,
    ImageLoaded(Result<image::DynamicImage, io::LoadError>),
    Backend(backend::Output),
    DetectionStarted,
    DetectionFinished,
    SelectModel(backend::ModelType),

    GoToScreen(Screen),
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
            Task::perform(
                async {
                    // Wait for some time to simulate loading
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    // image::load_from_memory(DEFAULT_IMAGE)
                    Err(io::LoadError::FileError)
                },
                |image| {
                    if let Ok(image) = image {
                        Message::ImageLoaded(Ok(image))
                    } else {
                        Message::ImageLoaded(Err(io::LoadError::FileError))
                    }
                },
            )
            .chain(Task::done(Message::GoToScreen(Screen::Inference))),
        )
    }

    pub fn title(&self) -> String {
        "ZeroShotRust".to_string()
    }

    fn send_to_backend(&self, message: backend::Input) -> Task<Message> {
        if let Some(tx) = self.backend_tx.clone() {
            let mut tx = tx.clone();
            return Task::perform(
                async move {
                    if let Err(e) = tx.send(message).await {
                        eprintln!("Failed to send message to backend: {:?}", e);
                    }
                },
                |_| Message::DetectionStarted,
            );
        }
        Task::none()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Backend(output) => match output {
                backend::Output::Loading => {
                    log::info!("Backend is loading...");
                    self.inference_state.model_info = Some("Loading model...".to_string());
                }
                backend::Output::Ready(tx) => {
                    log::info!("Backend is ready!");
                    self.backend_tx = Some(tx.clone());
                    self.inference_state.model_info = None;
                    self.screen = Screen::Inference;
                }
                backend::Output::ModelLoaded(model) => {
                    log::info!("Model loaded: {:?}", model);
                    self.inference_state.model_info = Some(format!("Model loaded: {:?}", model));
                }
                backend::Output::Progress(progress) => {
                    if progress >= 1.0 {
                        return Task::done(Message::DetectionFinished);
                    };
                }
                backend::Output::Finished(results) => {
                    log::info!("Detection finished!");
                    todo!("Handle detection results");
                }
                _ => todo!("Handle other backend outputs"),
            },
            Message::Detect(image) => {
                log::debug!("Button pressed!");
                return self.send_to_backend(Input::ProcessImage(image));
            }
            Message::LoadImage => {
                log::debug!("Load Image button pressed!");

                // Open file dialog to load an image
                self.inference_state.selecting_image = true;

                return Task::perform(io::open_image(), Message::ImageLoaded);
            }
            Message::ImageLoaded(result) => {
                self.inference_state.selecting_image = false;

                if let Ok(image) = result {
                    log::info!("Image loaded successfully!");

                    // TODO: make InferenceState::set_image or something instead
                    // let image_handle = iced::widget::image::Handle::from_bytes(DEFAULT_IMAGE.to_vec());
                    // self.inference_state.image = inference::Image::new(Some(image_handle));

                    // Convert DynamicImage to iced image
                    // let rgba = image.to_rgba8();
                    // let handle = iced::advanced::image::Handle::from_rgba(
                    //     rgba.width(),
                    //     rgba.height(),
                    //     rgba.as_raw().to_vec(),
                    // );
                    self.inference_state.image = inference::Image::new(&image);

                    self.image = Some(Arc::new(image));
                } else {
                    log::error!("Failed to load image: {:?}", result);
                }
            }
            Message::GoToScreen(screen) => {
                log::info!("Switching to screen: {:?}", screen);
                self.screen = screen;
            }
            Message::DetectionStarted => {
                self.inference_state.busy = true;
            }
            Message::DetectionFinished => {
                self.inference_state.busy = false;
            }
            Message::SelectModel(model) => {
                self.inference_state.selected_model = Some(model.clone());
                self.inference_state.model_info = Some(format!("Loading model {:?}...", model));
                log::info!("Selected model: {:?}", model);
                return self.send_to_backend(Input::SelectModel(model));
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::Loading => screen::loading(),
            Screen::Inference => screen::inference::view(self),
        }
    }

    // fn backend_subscription(model: Option<backend::Models>) -> iced::Subscription<backend::Output> {
    //     Subscription::run_with_id("backend", backend::connect(model))
    // }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        let backend = Subscription::run(backend::connect).map(Message::Backend);

        Subscription::batch([backend])
    }
}
