use std::sync::Arc;

use anyhow::Result;
// use futures::{
//     channel::mpsc,
//     channel::mpsc::{Receiver, Sender},
//     stream, SinkExt, StreamExt,
// };
use image::DynamicImage;
// use std::{default, future::Future, sync::Arc};
// use iced::Result;
// use async_trait::async_trait;
use usls::{Annotator, DataLoader, GroundingDINO, Options};

use crate::backend::DetectionParams;

use super::{BoundingBox, DetectionModel, DetectionResults, Detections};

pub struct ONNXModel {
    // receiver: Receiver<Input>,
    // sender: Sender<Output>,
    // Detection model and other resources
    // params: DetectionParams,
    model: usls::models::GroundingDINO,
}

impl Default for ONNXModel {
    fn default() -> Self {
        ONNXModel::new(&DetectionParams::default())
    }
}

impl DetectionModel for ONNXModel {
    fn new(parameters: &DetectionParams) -> Self {
        let class_names = parameters.class_names.clone();
        let options = Options::grounding_dino()
            .with_model_file(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/weights/grounding-dino/swint-ogc.onnx"
            ))
            // .with_model_file("./weights/grounding-dino/swint-ogc-fp16.onnx")
            // .with_model_dtype(args.dtype.as_str().try_into()?) // remember to download weights if you change dtype
            // .with_model_device(args.device.as_str().try_into()?)
            // .with_text_names(&args.labels.iter().map(|x| x.as_str()).collect::<Vec<_>>())
            .with_text_names(&class_names.iter().map(|x| x.as_str()).collect::<Vec<_>>())
            .with_class_confs(&[parameters.confidence_threshold])
            .with_text_confs(&[parameters.confidence_threshold])
            .commit()
            .expect("Failed to create options");

        log::info!("Creating model with options: {:?}", options);
        let model = GroundingDINO::new(options).expect("Failed to create model");
        log::info!("Model initialized");

        ONNXModel {
            // params: DetectionParams {
            //     confidence_threshold: 0.5,
            //     overlap_threshold: 0.5,
            //     class_names: class_names.clone(),
            // },
            model,
        }
    }

    fn detect(&mut self, image: &DynamicImage) -> Result<DetectionResults> {
        let xs = vec![image.clone()];
        let ys = self.model.forward(&xs)?;

        let annotator = Annotator::default()
            .with_bboxes_thickness(4)
            .with_saveout(self.model.spec());
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
