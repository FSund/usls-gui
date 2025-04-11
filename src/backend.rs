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
use usls::{models::GroundingDINO, Annotator, DataLoader, Options};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Models {
    Mock,
    GroundingDINO,
}

impl std::fmt::Display for Models {
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
    Ready(Sender<Input>),
    DetectionResults(Vec<Detection>),
    Progress(f32),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Detection {
    class: String,
    confidence: f32,
    bounding_box: BoundingBox,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone)]
pub struct DetectionParams {
    confidence_threshold: f32,
    overlap_threshold: f32,
    // Other parameters
    class_names: Vec<String>,
}

pub struct Backend {
    // receiver: Receiver<Input>,
    // sender: Sender<Output>,
    // Detection model and other resources
    params: DetectionParams,
    model: GroundingDINO,
}

impl Default for Backend {
    fn default() -> Self {
        let class_names = vec![
            "person".to_string(),
            "car".to_string(),
            "bus".to_string(),
            // Add more class names as needed
        ];

        let options = Options::grounding_dino()
            .with_model_file("./weights/grounding-dino/swint-ogc.onnx")
            // .with_model_file("./weights/grounding-dino/swint-ogc-fp16.onnx")
            // .with_model_dtype(args.dtype.as_str().try_into()?) // remember to download weights if you change dtype
            // .with_model_device(args.device.as_str().try_into()?)
            // .with_text_names(&args.labels.iter().map(|x| x.as_str()).collect::<Vec<_>>())
            .with_text_names(&class_names.iter().map(|x| x.as_str()).collect::<Vec<_>>())
            .with_class_confs(&[0.25])
            .with_text_confs(&[0.25])
            .commit()
            .expect("Failed to create options");

        log::info!("Creating model with options: {:?}", options);
        let model = GroundingDINO::new(options).expect("Failed to create model");
        log::info!("Model initialized");

        Backend {
            params: DetectionParams {
                confidence_threshold: 0.5,
                overlap_threshold: 0.5,
                class_names: class_names.clone(),
            },
            model,
        }
    }
}

impl Backend {
    pub fn new() -> Self {
        Backend {
            ..Default::default()
        }
    }

    // pub async fn run(&mut self) {
    //     loop {
    //         match self.receiver.next().await {
    //             Some(msg) => {
    //                 log::debug!("Received message: {:?}", msg);
    //                 match msg {
    //                     Input::ProcessImage(image_data) => {
    //                         // Process the image data
    //                         match self.process_image(&image_data).await {
    //                             Ok(detections) => {
    //                                 // Send results to frontend
    //                                 let _ = self
    //                                     .sender
    //                                     .send(Output::DetectionResults(detections))
    //                                     .await;
    //                             }
    //                             Err(e) => {
    //                                 // Handle error
    //                                 let _ = self.sender.send(Output::Error(e.to_string())).await;
    //                             }
    //                         }
    //                     }
    //                     Input::UpdateParams(_params) => {
    //                         // Update detection parameters
    //                         // ...
    //                     }
    //                     Input::Stop => {
    //                         break; // Stop processing
    //                     }
    //                 }
    //             }
    //             None => {
    //                 // Channel closed
    //                 log::debug!("Receiver channel closed");
    //                 break;
    //             }
    //         }
    //     }
    // }

    async fn process_image(
        &mut self,
        image_data: &Arc<DynamicImage>,
        sender: Sender<Output>,
    ) -> Result<Vec<Detection>> {
        // Perform object detection
        // This is where your ML model or algorithm would run

        let mut sender = sender.clone();

        // For progress updates during long operations:
        sender.send(Output::Progress(0.3)).await?;

        // Simulate CPU-intensive processing with a blocking sleep
        tokio::task::spawn_blocking(|| {
            log::debug!("Simulating CPU-intensive work for 10 seconds");
            std::thread::sleep(std::time::Duration::from_secs(10));

            // Any CPU-intensive work would go here
            // ...
        })
        .await?;
        log::debug!("Work completed!");

        let image = image_data.as_ref().clone();

        log::info!("Processing image");
        let ys = self.model.forward(&[image])?;

        // ... more processing ...
        sender.send(Output::Progress(0.7)).await?;

        // Return detected objects
        Ok(vec![/* detected objects */])
    }
}

/// Creates a new [`Stream`] that produces the items sent from a [`Future`]
/// to the [`mpsc::Sender`] provided to the closure.
///
/// This is a more ergonomic [`stream::unfold`], which allows you to go
/// from the "world of futures" to the "world of streams" by simply looping
/// and publishing to an async channel from inside a [`Future`].
pub fn channel<T, F>(size: usize, f: impl FnOnce(Sender<T>) -> F) -> impl stream::Stream<Item = T>
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
                    backend
                        .process_image(&image, output.clone())
                        .await
                        .expect("Failed to process image");

                    // Finally, we can optionally produce a message to tell the
                    // `Application` the work is done
                    output
                        .send(Output::Progress(1.0))
                        .await
                        .expect("Failed to send progress");

                    output
                        .send(Output::DetectionResults(vec![]))
                        .await
                        .expect("Failed to send detection results");
                }
                Input::UpdateParams(params) => {
                    backend.params = params;
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
