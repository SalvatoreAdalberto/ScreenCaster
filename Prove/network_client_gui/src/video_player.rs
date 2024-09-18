use iced::Command;
use iced::widget::image::Handle;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::child::FfmpegChild;
use tokio::sync::{Mutex as tokio_Mutex, mpsc};
use std::sync::{Arc, Mutex};
use image::{ImageBuffer, Rgb};
use tokio::task::JoinHandle;
use iced_futures::subscription::{self, Subscription};
use std::io::Read;


pub struct VideoPlayer {
   
   pub video_frame: Arc<Mutex<Option<Handle>>>, 
    state: PlayerState,
    decoder_handle: Option<JoinHandle<()>>,
    processing_handle: Option<JoinHandle<()>>,
    frame_receiver: Arc<tokio_Mutex<Option<mpsc::Receiver<Vec<u8>>>>>,
}


enum PlayerState {
    Waiting,
    Playing,
}

impl VideoPlayer {
    
    pub fn new() -> Self {
        VideoPlayer {
            video_frame: Arc::new(Mutex::new(None)),  
            state: PlayerState::Waiting,              
            decoder_handle: None,                     
            processing_handle: None,
            frame_receiver: Arc::new(tokio_Mutex::new(None)),                       
        }
    }

    fn frame_subscription(frame_receiver: Arc<tokio_Mutex<Option<mpsc::Receiver<Vec<u8>>>>>, video_frame: Arc<Mutex<Option<Handle>>>) -> Subscription<Message> {
        subscription::unfold(
            "FrameSubscription",     
            (frame_receiver, video_frame) ,             
            | ( frame_receiver, video_frame) | async move {
                
                
                

                
                
                let mut receiver_guard = frame_receiver.lock().await;
                if let Some(receiver) = receiver_guard.as_mut() {
                    if let Some(new_frame_data) = receiver.recv().await {
                            if let Ok(handle) = FrameProcessor::process_frame(new_frame_data, 640, 480) {
                                let mut frame_lock = video_frame.lock().unwrap();
                                *frame_lock = Some(handle);
                            }
                        }
                }
                drop(receiver_guard);
                
                (Message::FrameUpdate(Arc::clone(&video_frame)), (frame_receiver, video_frame))
            },
        )
    }
    
    
    pub fn subscription(&self) -> Subscription<Message> {
        match self.state {
            PlayerState::Playing => VideoPlayer::frame_subscription(Arc::clone(&self.frame_receiver), Arc::clone(&self.video_frame)),
            PlayerState::Waiting => Subscription::none(),
        }
    }

    
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Play => {
                let _ = self.play();
            }
            Message::Stop => {
                self.stop();
            }
            Message::FrameUpdate(frame) => {
                
                self.video_frame = frame;

            }
        }

        Command::none()
    }
    
    fn play(&mut self) -> Command<Message> {
        if let PlayerState::Playing = self.state {
            return Command::none(); 
        }

        
        let (frame_sender, frame_receiver) = mpsc::channel(10);
        
        self.frame_receiver = Arc::new(tokio_Mutex::new(Some(frame_receiver)));
        
        let decoder_handle = tokio::spawn(async move {
            let mut video_decoder = VideoDecoder::start("udp://127.0.0.1:1235", frame_sender);
            println!("Video decoder started");
            video_decoder.read_frames().await;
        });

        
        self.decoder_handle = Some(decoder_handle);

        
        self.state = PlayerState::Playing;

        Command::none()
    }

    
    fn stop(&mut self) {
         
         if let Some(decoder_handle) = self.decoder_handle.take() {
            decoder_handle.abort(); 
        }

        
        if let Some(processing_handle) = self.processing_handle.take() {
            processing_handle.abort(); 
        }

        
        let mut frame_lock = self.video_frame.lock().unwrap();
        *frame_lock = None;

        
        self.state = PlayerState::Waiting;
    }
}


pub struct VideoDecoder {
    
    ffmpeg_process: Option<FfmpegChild>,
    
    
    frame_sender: mpsc::Sender<Vec<u8>>,
}

impl VideoDecoder {
    
    pub fn start(stream_url: &str, frame_sender: mpsc::Sender<Vec<u8>>) -> Self {
        let  ffmpeg_process = FfmpegCommand::new()
            .input(stream_url)
            .output("pipe:1") 
            .args(&["-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
            .spawn()
            .expect("Failed to spawn ffmpeg");

        VideoDecoder {
            ffmpeg_process: Some(ffmpeg_process),
            frame_sender,
        }
    }

    
    pub async fn read_frames(&mut self) {
        if let Some(ref mut process) = self.ffmpeg_process {
            let mut stdout = process.take_stdout().expect("No stdout");
            let mut buffer = vec![0u8; 640 * 480 * 3]; 

            while stdout.read_exact(&mut buffer).is_ok() {
                
                self.frame_sender.send(buffer.clone()).await.unwrap();
            }
        }
    }

    
    
    
    
    
    
}

pub struct FrameProcessor;

impl FrameProcessor {
    
    pub fn process_frame(frame: Vec<u8>, width: u32, height: u32) -> Result<Handle, ()> {
        let img_buffer = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, frame).unwrap();
        let png_data = image::DynamicImage::ImageRgb8(img_buffer).to_rgba8();
        Ok(Handle::from_memory(png_data.to_vec()))
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Play,               
    Stop,               
    FrameUpdate(Arc<Mutex<Option<Handle>>>), 
}