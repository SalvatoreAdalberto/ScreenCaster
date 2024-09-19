use iced::Command;
use iced::widget::image::Handle;
use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent, event::OutputVideoFrame};
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
    frame_receiver: Arc<tokio_Mutex<Option<mpsc::UnboundedReceiver<Handle>>>>,
}

//Added working network_server (recording and sending), working network_client (receiving and storing as mp4) and not working network_client_gui (gui seems to work, no compilation errors, no runtime errors playing/stopping, but video does not appear)
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

    fn frame_subscription(frame_receiver: Arc<tokio_Mutex<Option<mpsc::UnboundedReceiver<Handle>>>>, video_frame: Arc<Mutex<Option<Handle>>>) -> Subscription<Message> {
        subscription::unfold(
            "FrameSubscription",     
            (frame_receiver, video_frame) ,             
            | ( frame_receiver, video_frame) | async move {
                let mut receiver_guard = frame_receiver.lock().await;
                if let Some(receiver) = receiver_guard.as_mut() {
                    let handle =  receiver.recv().await.unwrap();
                    let mut frame_lock = video_frame.lock().unwrap();
                    *frame_lock = Some(handle);
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
                //println!("4 Updated frame data");
                self.video_frame = frame;

            }
        }

        Command::none()
    }
    
    fn play(&mut self) -> Command<Message> {
        if let PlayerState::Playing = self.state {
            return Command::none(); 
        }
        let (frame_sender, frame_receiver) = mpsc::unbounded_channel();
        self.frame_receiver = Arc::new(tokio_Mutex::new(Some(frame_receiver)));
        let decoder_handle = tokio::spawn(async move {
            VideoDecoder::start("udp://127.0.0.1:1235", frame_sender);
            println!("0 Video decoder started");
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
}

impl VideoDecoder {
    
    pub fn start(stream_url: &str, frame_sender: mpsc::UnboundedSender<Handle>) {
        FfmpegCommand::new()
        .format("mpegts")
        .input(stream_url)
        .format("rawvideo")
        .args(&["-pix_fmt", "rgb24"])
        .output("pipe:1")
        .spawn()
        .unwrap()
        .iter()
        .unwrap()
        .filter_frames()
        .for_each(|f| {
            let sender_clone = frame_sender.clone();
            tokio::spawn( async move{
                if let Ok(handle) = VideoDecoder::process_frame(f.data, f.width, f.height) {
                    sender_clone.send(handle).unwrap();
                }else{
                    println!("Error processing frame");
                }
            });
        });

    }

    pub fn process_frame(frame: Vec<u8>, width: u32, height: u32) -> Result<Handle, ()> {
        let img_buffer = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, frame).unwrap();
        let png_data = image::DynamicImage::ImageRgb8(img_buffer).to_rgba8();
        unsafe {
            if COUNTER % 10 == 0 {
                let c = format!("out_{}.png", COUNTER);   
                png_data.save(c).unwrap();
            }
            COUNTER += 1;
        }
        Ok(Handle::from_memory(png_data.to_vec()))
    }

}

static mut COUNTER: i32 = 0;

#[derive(Debug, Clone)]
pub enum Message {
    Play,               
    Stop,               
    FrameUpdate(Arc<Mutex<Option<Handle>>>), 
}


            //Read each packet at a time packing them in mpeg1 frames
            // let mut stdout = process.take_stdout().expect("No stdout");
            // let mut buffer1 = [0u8; 1024]; 
            // let mut buffer2: Vec<u8> = Vec::new();

            // loop {
            //     let bytes_read = stdout.read(&mut buffer1).unwrap();
            //     if bytes_read == 0 {
            //         break;
            //     }
            //     buffer2.extend_from_slice(&buffer1[..bytes_read]);

            //     if let Some(start) = buffer2.iter().position(|&b| b == 0x47){
            //         buffer2.drain(..start);
            //         if let Some(end) = buffer2[start+1..].iter().position(|&b| b == 0x47){
            //             let frame = buffer2.drain(..end).collect::<Vec<u8>>();
            //             self.frame_sender.send(frame).await.unwrap();
            //         }else{
            //             //No ending 0x47 found -> continue reading
            //             continue;
            //         }
            //     }else{
            //         //No starting 0x47 found -> continue reading
            //         continue;
            //     }

            // }
               
                