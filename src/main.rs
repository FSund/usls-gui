mod backend;
mod logging;
use backend::{Backend, Input, Output};

use anyhow::{Context, Result};
use futures::{SinkExt, Stream, StreamExt};
use iced::widget::{button, center, checkbox, column, row, text};
use iced::{Element, Font, Subscription, Task};
// use std::sync::mpsc::{self, Receiver, Sender};
// use tokio::sync::mpsc;
use futures::channel::mpsc;
// use std::sync::{Arc, Mutex};
use std::thread;
// use tracing::instrument::WithSubscriber;

pub fn main() -> Result<()> {
    logging::init_logging()?;
    log::info!("Starting the application...");

    // Create channels for communication
    let (backend_tx, backend_rx) = mpsc::channel(16);
    let (frontend_tx, frontend_rx) = mpsc::channel(16);

    // Launch backend in a separate thread
    let _backend_handle = thread::spawn(move || {
        // Create a new Tokio runtime for this thread
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

        // Run the backend within the Tokio runtime
        runtime.block_on(async {
            let mut backend = Backend::new(backend_rx, frontend_tx);
            backend.run().await;
        });
    });

    iced::application(Example::title, Example::update, Example::view)
        .subscription(Example::subscription)
        .run_with(|| (Example::new(backend_tx, frontend_rx), Task::none()))
        .context("Failed to run the application")
}

#[derive(Default)]
struct Example {
    default: bool,
    styled: bool,
    custom: bool,

    // Add channels for communication
    backend_tx: Option<mpsc::Sender<Input>>,
    frontend_rx: Option<mpsc::Receiver<Output>>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ButtonPressed,
}

impl Example {
    fn new(backend_tx: mpsc::Sender<Input>, frontend_rx: mpsc::Receiver<Output>) -> Self {
        Self {
            backend_tx: Some(backend_tx),
            frontend_rx: Some(frontend_rx),
            ..Default::default()
        }
    }

    fn title(&self) -> String {
        "Iced Example".to_string()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ButtonPressed => {
                log::debug!("Button pressed!");

                // Send message to backend
                if let Some(tx) = &self.backend_tx {
                    let msg = Input::ProcessImage(vec![0; 1024]); // Example image data

                    // Spawn a new task to send the message
                    let mut tx = tx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tx.send(msg).await {
                            eprintln!("Failed to send message to backend: {:?}", e);
                        }
                    });
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let button = button("Do the thing").on_press(Message::ButtonPressed);

        let content = column![button];

        center(content).into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
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

        Subscription::run()
    }
}

fn connect(rx: mpsc::Receiver<Output>) -> impl Stream<Item = Message> {
    async_stream::stream! {
        let mut rx = rx;
        while let Some(output) = rx.next().await {
            match output {
                Output::DetectionResults(_) => yield Message::ButtonPressed,
                Output::Error(_) => yield Message::ButtonPressed,
                Output::Progress(_) => yield Message::ButtonPressed,
                // Add appropriate message mappings based on the outputs
            }
        }
    }
}

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
