use anyhow::Result;
use image::DynamicImage;
use usls::{models::GroundingDINO, Annotator, DataLoader, Options};

use super::{BoundingBox, DetectionModel, DetectionParams, DetectionResults, Detections};

impl DetectionModel for GroundingDINO {
    fn detect(&mut self, image: &DynamicImage) -> Result<DetectionResults> {
        let xs = vec![image.clone()];
        let ys = self.forward(&xs)?;

        let annotator = Annotator::default()
            .with_bboxes_thickness(4)
            .with_saveout(self.spec());
        let annotated = annotator.plot(&xs, &ys, false)?;

        Ok(DetectionResults {
            // y: ys[0].clone(),
            annotated: annotated.first().expect("No annotated image found").clone(),
            detections: ys[0].clone().into(),
        })
    }
}

// usls::Y into DetectionResults
impl From<usls::Y> for Detections {
    fn from(y: usls::Y) -> Self {
        let mut boxes = Vec::new();
        for bbox in y.bboxes().expect("Failed to get bounding boxes") {
            boxes.push(BoundingBox {
                class: bbox.name().unwrap_or("unknown").to_string(),
                confidence: bbox.confidence(),
                x: bbox.x(),
                y: bbox.y(),
                width: bbox.w(),
                height: bbox.h(),
            });
        }
        Detections { boxes }
    }
}
