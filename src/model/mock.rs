use super::{DetectionModel, DetectionResults, Detections};
use crate::backend::DetectionParams;

use anyhow::Result;
use image::DynamicImage;

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
