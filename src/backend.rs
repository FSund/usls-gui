use anyhow::{Context, Result};
use futures::{
    channel::mpsc,
    channel::mpsc::{Receiver, Sender},
    stream, SinkExt, StreamExt,
};
use image::DynamicImage;
use std::{future::Future, sync::Arc};

pub use crate::model::ModelType;
use crate::model::{mock, onnx, DetectionModel, DetectionResults};

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
    Progress(f32),
    Finished(DetectionResults),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct DetectionParams {
    pub confidence_threshold: f32,
    pub class_names: Vec<String>,
}

impl Default for DetectionParams {
    fn default() -> Self {
        DetectionParams {
            confidence_threshold: 0.25,
            class_names: vec!["person".to_string(), "car".to_string(), "bus".to_string()],
        }
    }
}

struct Backend {
    params: DetectionParams,
    model: Option<Box<dyn DetectionModel>>,
    selected_model: Option<ModelType>,
}

impl Default for Backend {
    fn default() -> Self {
        Backend::new()
    }
}

impl Backend {
    pub fn new() -> Self {
        let params = DetectionParams::default();
        Backend {
            model: None,
            selected_model: None,
            params,
        }
    }

    async fn select_model(&mut self, model_type: ModelType) -> Result<()> {
        // Check if the model is already loaded
        if self.selected_model == Some(model_type.clone()) {
            log::info!("Model {model_type} is already loaded, skipping initialization.");
            return Ok(());
        }

        // Use tokio::task::spawn_blocking for CPU-bound operations
        // to avoid blocking the async runtime
        // (probably not necessary after we implemented deferred model loading)
        let params = self.params.clone();
        let model = tokio::task::spawn_blocking(move || {
            let model: Box<dyn DetectionModel> = match &model_type {
                ModelType::Mock => Box::new(mock::MockModel::new(&params)),
                ModelType::GroundingDINO => Box::new(onnx::ONNXModel::new(&params)),
            };
            model
        })
        .await
        .context("Failed to load model. Blocking task panicked.")?;
        self.model = Some(model);
        Ok(())
    }

    async fn process_image(
        &mut self,
        image_data: &Arc<DynamicImage>,
        sender: &Sender<Output>,
    ) -> Result<DetectionResults> {
        log::info!("Processing image");

        let mut sender = sender.clone();
        sender.send(Output::Progress(0.3)).await?;

        // Perform detection
        let results = if let Some(model) = &mut self.model {
            // Use block_in_place to run the blocking operation
            // on the current thread to avoid blocking the async runtime
            let results = tokio::task::block_in_place(|| model.detect(image_data.as_ref()))?;
            Ok(results)
        } else {
            Err(anyhow::anyhow!("Model not initialized"))
        };

        sender.send(Output::Progress(0.7)).await?;
        results
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

        let mut backend = Backend::new();

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
                    backend
                        .select_model(model_type.clone())
                        .await
                        .expect("Failed to select model");
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
