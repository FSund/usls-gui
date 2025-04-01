// use tokio::sync::mpsc::{Receiver, Sender};
use anyhow::Result;
use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};
// use iced::Result;

#[derive(Debug, Clone)]
pub enum BackendMsg {
    ProcessImage(Vec<u8>),
    UpdateParams(DetectionParams),
    Stop,
}

pub enum FrontendMsg {
    DetectionResults(Vec<Detection>),
    Progress(f32),
    Error(String),
}

pub struct Detection {
    class: String,
    confidence: f32,
    bounding_box: BoundingBox,
}

pub struct BoundingBox {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct DetectionParams {
    confidence_threshold: f32,
    overlap_threshold: f32,
    // Other parameters
}

pub struct Backend {
    receiver: Receiver<BackendMsg>,
    sender: Sender<FrontendMsg>,
    // Detection model and other resources
}

impl Backend {
    pub fn new(receiver: Receiver<BackendMsg>, sender: Sender<FrontendMsg>) -> Self {
        Backend {
            receiver,
            sender,
            // Initialize detection model and other resources
        }
    }

    pub async fn run(&mut self) {
        loop {
            match self.receiver.next().await {
                Some(msg) => {
                    log::debug!("Received message: {:?}", msg);
                    match msg {
                        BackendMsg::ProcessImage(image_data) => {
                            // Process the image data
                            match self.process_image(&image_data).await {
                                Ok(detections) => {
                                    // Send results to frontend
                                    let _ = self
                                        .sender
                                        .send(FrontendMsg::DetectionResults(detections))
                                        .await;
                                }
                                Err(e) => {
                                    // Handle error
                                    let _ =
                                        self.sender.send(FrontendMsg::Error(e.to_string())).await;
                                }
                            }
                        }
                        BackendMsg::UpdateParams(_params) => {
                            // Update detection parameters
                            // ...
                        }
                        BackendMsg::Stop => {
                            break; // Stop processing
                        }
                    }
                }
                None => break, // Channel closed
            }
        }
    }

    async fn process_image(&self, _image_data: &[u8]) -> Result<Vec<Detection>> {
        // Perform object detection
        // This is where your ML model or algorithm would run

        let mut sender = self.sender.clone();

        // For progress updates during long operations:
        sender.send(FrontendMsg::Progress(0.3)).await?;

        // Simulate CPU-intensive processing with a blocking sleep
        tokio::task::spawn_blocking(|| {
            log::debug!("Simulating CPU-intensive work for 10 seconds");
            std::thread::sleep(std::time::Duration::from_secs(10));

            // Any CPU-intensive work would go here
            // ...
        })
        .await?;
        log::debug!("Work completed!");

        // ... more processing ...
        sender.send(FrontendMsg::Progress(0.7)).await?;

        // Return detected objects
        Ok(vec![/* detected objects */])
    }
}
