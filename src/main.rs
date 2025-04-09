mod backend;
mod frontend;
mod io;
mod logging;

// use backend::{Backend, Input, Output};
use frontend::Example;

use anyhow::Context;
// use futures::{SinkExt, Stream, StreamExt};
// use iced::widget::{button, center, checkbox, column, container, row, text, Space};
// use iced::{Element, Font, Length, Subscription, Task};
// use std::sync::mpsc::{self, Receiver, Sender};
// use tokio::sync::mpsc;
// use futures::channel::mpsc;
// use image::DynamicImage;
// use std::sync::{Arc, Mutex};
// use std::thread;
// use tracing::instrument::WithSubscriber;

pub fn main() -> anyhow::Result<()> {
    logging::init_logging()?;
    log::info!("Starting the application...");

    iced::application(Example::title, Example::update, Example::view)
        .subscription(Example::subscription)
        .run_with(Example::new)
        .context("Failed to run the application")
}
