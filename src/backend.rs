use anyhow::Result;
use futures::{
    channel::mpsc,
    channel::mpsc::{Receiver, Sender},
    stream, SinkExt, StreamExt,
};
use image::DynamicImage;
use std::{future::Future, sync::Arc};

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
    SelectModel(ModelType),
    UpdateParams(DetectionParams),
    Stop,
}

#[derive(Debug, Clone)]
pub enum Output {
    Loading,
    Ready(Sender<Input>),
    ModelLoaded(ModelType),
    Progress(f32),
    Finished(DetectionResults),
    Error(String),
}

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
    model: Option<Box<dyn DetectionModel>>,
}

impl Default for Backend {
    fn default() -> Self {
        Backend::new(None)
    }
}

impl Backend {
    pub fn new(model_type: Option<ModelType>) -> Self {
        let params = DetectionParams::default();
        let model = match model_type {
            Some(model_type) => {
                let model: Box<dyn DetectionModel> = match model_type {
                    ModelType::Mock => Box::new(mock::MockModel::new(&params)),
                    ModelType::GroundingDINO => Box::new(onnx::ONNXModel::new(&params)),
                };
                Some(model)
            }
            None => None,
        };

        Backend { model }
    }

    async fn process_image(
        &mut self,
        image_data: &Arc<DynamicImage>,
        sender: &Sender<Output>,
    ) -> Result<DetectionResults> {
        log::info!("Processing image");

        if let Some(model) = &mut self.model {
            // Use the model to process the image
            let mut sender = sender.clone();
            sender.send(Output::Progress(0.3)).await?;
            let results = tokio::task::block_in_place(|| model.detect(image_data.as_ref()))?;
            sender.send(Output::Progress(0.7)).await?;
            Ok(results)
        } else {
            Err(anyhow::anyhow!("Model not initialized"))
        }
    }

    fn update_params(&mut self, params: DetectionParams) {
        // Update detection parameters
        todo!("Implement parameter update logic");
        log::info!("Updated detection parameters: {:?}", params);
    }
}

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

pub fn connect() -> impl futures::stream::Stream<Item = Output> {
    channel(100, |mut output| async move {
        // Create channel
        let (sender, mut receiver) = mpsc::channel(100);

        output
            .send(Output::Loading)
            .await
            .expect("Failed to send loading");

        let mut backend = Backend::new(None);

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
                Input::SelectModel(model_type) => {
                    backend = Backend::new(Some(model_type.clone()));
                    output
                        .send(Output::ModelLoaded(model_type))
                        .await
                        .expect("Failed to send model loaded");
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
