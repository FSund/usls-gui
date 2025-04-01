mod backend;
mod logging;
use backend::{Backend, BackendMsg, FrontendMsg};

use anyhow::{Context, Result};
use iced::widget::{button, center, checkbox, column, row, text};
use iced::{Element, Font, Task};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

const ICON_FONT: Font = Font::with_name("icons");

pub fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339())
        .init();
    logging::init_logging()?;

    // Create channels for communication
    let (backend_tx, backend_rx) = mpsc::channel();
    let (frontend_tx, frontend_rx) = mpsc::channel();

    // Launch backend in a separate thread
    let backend_handle = thread::spawn(move || {
        let mut backend = Backend::new(backend_rx, frontend_tx);
        backend.run();
    });

    iced::application("Checkbox - Iced", Example::update, Example::view)
        .font(include_bytes!("../assets/icons.ttf").as_slice())
        .run_with(|| (Example::new(backend_tx, frontend_rx), Task::none()))
        .context("Failed to run the application")
}

#[derive(Default)]
struct Example {
    default: bool,
    styled: bool,
    custom: bool,

    // Add channels for communication
    backend_tx: Option<Sender<BackendMsg>>,
    frontend_rx: Option<Receiver<FrontendMsg>>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ButtonPressed,
}

impl Example {
    fn new(backend_tx: Sender<BackendMsg>, frontend_rx: Receiver<FrontendMsg>) -> Self {
        Self {
            backend_tx: Some(backend_tx),
            frontend_rx: Some(frontend_rx),
            ..Default::default()
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ButtonPressed => {
                log::debug!("Button pressed!");
                println!("wat");

                // Send message to backend
                if let Some(tx) = &self.backend_tx {
                    let msg = BackendMsg::ProcessImage(vec![0; 1024]); // Example image data
                    tx.send(msg).unwrap();
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let button = button("Do the thing").on_press(Message::ButtonPressed);

        let content = column![button];

        center(content).into()
    }
}
