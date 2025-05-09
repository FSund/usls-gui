use crate::{backend::DetectionParams, model::DetectionModel};

use anyhow::Result;
use futures::{
    channel::mpsc,
    channel::mpsc::{Receiver, Sender},
    stream, SinkExt, StreamExt,
};
use image::DynamicImage;
use std::{default, future::Future, sync::Arc};
// use iced::Result;
use async_trait::async_trait;
use usls::{Annotator, DataLoader, Options};

use super::{DetectionResults, Detections};

#[derive(Default)]
pub struct MockModel {}


impl DetectionModel for MockModel {
    fn new(_parameters: &DetectionParams) -> Self {
        MockModel::default()
    }

    fn detect(&mut self, image_data: &DynamicImage) -> Result<DetectionResults> {
        // async move {
        // let mut sender = sender.clone();

        // Simulate some processing
        // sender.send(Output::Progress(0.3)).await?;
        // tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        // sender.send(Output::Progress(0.7)).await?;

        // Simulate detection
        // let detections = vec![
        //

        // Return mock results
        let results = DetectionResults {
            detections: Detections { boxes: vec![] },
            annotated: image_data.clone(),
        };

        Ok(results)
        // }
    }
}
