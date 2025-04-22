use anyhow::Result;
use image::DynamicImage;

use super::{BoundingBox, DetectionModel, DetectionParams, DetectionResults, Detections};

#[derive(Clone)]
pub struct MockModel {
    params: DetectionParams,
}

impl MockModel {
    pub fn new(params: &DetectionParams) -> Result<Self> {
        Ok(MockModel {
            params: params.clone(),
        })
    }
}

impl DetectionModel for MockModel {
    fn detect(&mut self, image: &DynamicImage) -> Result<DetectionResults> {
        // Create a mock detection result
        let annotated = image.clone();

        // Create mock detections
        let bboxes = vec![
            BoundingBox {
                class: "mock_class".to_string(),
                confidence: 0.9,
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 200.0,
            },
            BoundingBox {
                class: "mock_class_2".to_string(),
                confidence: 0.8,
                x: 30.0,
                y: 40.0,
                width: 150.0,
                height: 250.0,
            },
        ];

        let detections = Detections { boxes: bboxes };

        Ok(DetectionResults {
            annotated,
            detections,
        })
    }
}
