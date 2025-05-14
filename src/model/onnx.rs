use anyhow::Result;
use image::DynamicImage;
use usls::{Annotator, DataLoader, GroundingDINO, Options};

use super::{BoundingBox, DetectionModel, DetectionResults, Detections, ModelType};
use crate::backend::DetectionParams;

pub struct ONNXModel {
    // receiver: Receiver<Input>,
    // sender: Sender<Output>,
    // Detection model and other resources
    params: DetectionParams,
    model: Option<usls::models::GroundingDINO>,
}

impl Default for ONNXModel {
    fn default() -> Self {
        ONNXModel::new(&DetectionParams::default())
    }
}

impl ONNXModel {
    fn get_model(&mut self) -> &mut usls::models::GroundingDINO {
        // Defer model loading until first use (lazy loading)
        if self.model.is_none() {
            let class_names = self.params.class_names.clone();
            let options = Options::grounding_dino()
                .with_model_file(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/weights/grounding-dino/swint-ogc.onnx"
                ))
                .with_text_names(&class_names.iter().map(|x| x.as_str()).collect::<Vec<_>>())
                .with_class_confs(&[self.params.confidence_threshold])
                .with_text_confs(&[self.params.confidence_threshold])
                .commit()
                .expect("Failed to create options");

            log::info!("Creating model with options: {:?}", options);
            let model = GroundingDINO::new(options).expect("Failed to create model");
            log::info!("Model initialized");
            self.model = Some(model);
        }
        self.model.as_mut().expect("Model not loaded")
    }
}

impl DetectionModel for ONNXModel {
    fn new(parameters: &DetectionParams) -> Self {
        ONNXModel {
            model: None,
            params: parameters.clone(),
        }
    }

    fn detect(&mut self, image: &DynamicImage) -> Result<DetectionResults> {
        let model = self.get_model();
        let xs = vec![image.clone()];
        let ys = model.forward(&xs)?;

        let annotator = Annotator::default()
            .with_bboxes_thickness(4)
            .with_saveout(model.spec());
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
