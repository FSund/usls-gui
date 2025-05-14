pub mod mock;
pub mod onnx;

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

use crate::backend::DetectionParams;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelType {
    Mock,
    GroundingDINO,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    class: String,
    confidence: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone)]
pub struct Detections {
    boxes: Vec<BoundingBox>,
}

#[derive(Debug, Clone)]
pub struct DetectionResults {
    detections: Detections,
    // y: usls::Y,
    annotated: DynamicImage,
}

pub trait DetectionModel: Send {
    fn detect(&mut self, image: &DynamicImage) -> Result<DetectionResults>;
    fn new(parameters: &DetectionParams) -> Self
    where
        Self: Sized;
    // fn update_params(&mut self, params: &DetectionParams) -> Result<()>;
    // fn clone_box(&self) -> Box<dyn DetectionModel>;
    // fn model_type(&self) -> ModelType;
}
