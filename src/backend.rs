// use tokio::sync::mpsc::{Receiver, Sender};
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
// use usls::{models::GroundingDINO, Annotator, DataLoader, Options};

use crate::model::{mock, onnx, DetectionModel, DetectionResults};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelType {
    Mock,
    GroundingDINO,
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Mock => "Mock",
            Self::GroundingDINO => "GroundingDINO",
        })
    }
}

#[derive(Debug, Clone)]
pub enum Input {
    ProcessImage(Arc<DynamicImage>),
    UpdateParams(DetectionParams),
    Stop,
}

#[derive(Debug, Clone)]
pub enum Output {
    Loading,
    Ready(Sender<Input>),
    Progress(f32),
    Finished(DetectionResults),
    Error(String),
}

// #[derive(Debug, Clone)]
// pub struct DetectionResults {
//     // detections: Vec<Detection>,
//     y: usls::Y,
//     annotated: DynamicImage,
// }

// #[derive(Debug, Clone)]
// pub struct Detection {
//     class: String,
//     confidence: f32,
//     bounding_box: BoundingBox,
// }

// #[derive(Debug, Clone)]
// pub struct BoundingBox {
//     x: f32,
//     y: f32,
//     width: f32,
//     height: f32,
// }

#[derive(Debug, Clone)]
pub struct DetectionParams {
    pub confidence_threshold: f32,
    // overlap_threshold: f32,
    // Other parameters
    pub class_names: Vec<String>,
}

impl Default for DetectionParams {
    fn default() -> Self {
        DetectionParams {
            confidence_threshold: 0.25,
            // overlap_threshold: 0.5,
            class_names: vec!["person".to_string(), "car".to_string(), "bus".to_string()],
        }
    }
}

struct Backend {
    // receiver: Receiver<Input>,
    // sender: Sender<Output>,
    // Detection model and other resources
    // params: DetectionParams,
    model: Box<dyn DetectionModel>,
}

impl Default for Backend {
    fn default() -> Self {
        Backend::new(ModelType::Mock)
    }
}

impl Backend {
    pub fn new(model_type: ModelType) -> Self {
        let params = DetectionParams::default();
        let model: Box<dyn DetectionModel> = match model_type {
            ModelType::Mock => Box::new(mock::MockModel::new(&params)),
            ModelType::GroundingDINO => Box::new(onnx::ONNXModel::new(&params)),
        };

        Backend { model }
    }

    // async fn detect(
    //     &mut self,
    //     image_data: &Arc<DynamicImage>,
    //     sender: Sender<Output>,
    // ) -> Result<DetectionResults> {
    //     // async move {
    //     // Perform object detection
    //     // This is where your ML model or algorithm would run

    //     let mut sender = sender.clone();

    //     // For progress updates during long operations:
    //     sender.send(Output::Progress(0.3)).await?;

    //     // // Simulate CPU-intensive processing with a blocking sleep
    //     // tokio::task::spawn_blocking(|| {
    //     //     log::debug!("Simulating CPU-intensive work for 10 seconds");
    //     //     std::thread::sleep(std::time::Duration::from_secs(10));

    //     //     // Any CPU-intensive work would go here
    //     //     // ...
    //     // })
    //     // .await?;
    //     // log::debug!("Work completed!");

    //     let xs = vec![image_data.as_ref().clone()];

    //     sender.send(Output::Progress(0.7)).await?;

    //     log::info!("Processing image");
    //     let ys = tokio::task::block_in_place(|| self.model.forward(&xs))?;

    //     let annotator = Annotator::default()
    //         .with_bboxes_thickness(4)
    //         .with_saveout(self.model.spec());
    //     let annotated = annotator.plot(&xs, &ys, false)?;

    //     let results = DetectionResults {
    //         y: ys[0].clone(),
    //         annotated: annotated.first().expect("No annotated image found").clone(),
    //     };

    //     // Return detected objects
    //     Ok(results)
    //     // }
    // }

    async fn process_image(
        &mut self,
        image_data: &Arc<DynamicImage>,
        sender: &Sender<Output>,
    ) -> Result<DetectionResults> {
        log::info!("Processing image");

        let mut sender = sender.clone();
        sender.send(Output::Progress(0.3)).await?;
        let results = tokio::task::block_in_place(|| self.model.detect(image_data.as_ref()))?;
        sender.send(Output::Progress(0.7)).await?;

        Ok(results)
    }

    fn update_params(&mut self, params: DetectionParams) {
        // Update detection parameters
        todo!("Implement parameter update logic");
        log::info!("Updated detection parameters: {:?}", params);
    }
}

// pub struct MockBackend {
//     // Mock backend for testing
// }
// impl MockBackend {
//     pub fn new() -> Self {
//         MockBackend {}
//     }
// }

// #[async_trait]
// impl InferenceBackend for MockBackend {
//     async fn process_image(
//         &mut self,
//         image_data: &Arc<DynamicImage>,
//         sender: Sender<Output>,
//     ) -> Result<DetectionResults> {
//         // async move {
//         let mut sender = sender.clone();

//         // Simulate some processing
//         sender.send(Output::Progress(0.3)).await?;
//         tokio::time::sleep(std::time::Duration::from_secs(2)).await;
//         sender.send(Output::Progress(0.7)).await?;

//         // Return mock results
//         let results = DetectionResults {
//             y: usls::Y::default(),
//             annotated: image_data.as_ref().clone(),
//         };

//         Ok(results)
//         // }
//     }
// }

/// Creates a new [`Stream`] that produces the items sent from a [`Future`]
/// to the [`mpsc::Sender`] provided to the closure.
///
/// This is a more ergonomic [`stream::unfold`], which allows you to go
/// from the "world of futures" to the "world of streams" by simply looping
/// and publishing to an async channel from inside a [`Future`].
fn channel<T, F>(size: usize, f: impl FnOnce(Sender<T>) -> F) -> impl stream::Stream<Item = T>
where
    F: Future<Output = ()>,
{
    let (sender, receiver) = mpsc::channel(size);

    let runner = stream::once(f(sender)).filter_map(|_| async { None });

    stream::select(receiver, runner)
}

// async fn create_backend(model: &ModelType) -> Box<dyn InferenceBackend + Send> {
//     match model {
//         ModelType::Mock => Box::new(MockBackend::new()),
//         ModelType::GroundingDINO => {
//             let backend = tokio::task::spawn_blocking(|| Backend::new())
//                 .await
//                 .unwrap();
//             Box::new(backend)
//         }
//     }
// }

pub fn connect(model: ModelType) -> impl futures::stream::Stream<Item = Output> {
    channel(100, |mut output| async move {
        // Create channel
        let (sender, mut receiver) = mpsc::channel(100);

        output
            .send(Output::Loading)
            .await
            .expect("Failed to send loading");

        let mut backend = Backend::new(model);

        // Send the sender back to the application
        output
            .send(Output::Ready(sender))
            .await
            .expect("Failed to send sender");

        loop {
            // Read next input sent from `Application`
            let input = receiver.select_next_some().await;

            match input {
                Input::ProcessImage(image) => {
                    // Do some async work...
                    let results = backend
                        .process_image(&image, &output)
                        .await
                        .expect("Failed to process image");

                    // Finally, we can optionally produce a message to tell the
                    // `Application` the work is done
                    output
                        .send(Output::Progress(1.0))
                        .await
                        .expect("Failed to send progress");

                    output
                        .send(Output::Finished(results))
                        .await
                        .expect("Failed to send detection results");
                }
                Input::UpdateParams(params) => {
                    backend.update_params(params);
                }
                Input::Stop => {
                    // Stop processing
                    break;
                }
            }
        }
    })
}

// fn connect() -> impl futures::stream::Stream<Item = Output> {
//     let (sender, mut receiver): (Sender<Input>, Receiver<Input>) = mpsc::channel(100);
//     let backend = Backend::new();

//     async_stream::stream! {
//         let mut rx = receiver;
//         while let Some(input) = rx.next().await {
//             yield Output::Progress(0.0);
//         }
//     }
// }
