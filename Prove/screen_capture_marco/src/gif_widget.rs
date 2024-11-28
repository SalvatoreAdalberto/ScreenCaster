use iced::{
    time as iced_time, widget::image, widget::image::Handle, Command, Element, Length, Subscription, Theme
};
use gif::{Decoder, Frame};
use std::fs::File;
use std::time::Duration;
use iced::widget;


pub struct GifPlayer {
    frames: Vec<image::Handle>, // Store GIF frames as images
    current_frame: usize,
    frame_delay: Duration,
}

#[derive(Debug, Clone)]
pub enum GifPlayerMessage {
    NextFrame,
}

impl GifPlayer {
    
    pub fn new() -> Self {
        // Load GIF frames
        let (frames, frame_delay) = load_gif_frames("./loading49.gif");

        
            GifPlayer {
                frames,
                current_frame: 0,
                frame_delay,
            }
    
    }

    pub fn update(&mut self, message: GifPlayerMessage) -> Command<GifPlayerMessage> {
        match message {
            GifPlayerMessage::NextFrame => {
                self.current_frame = (self.current_frame + 1) % self.frames.len();
            }
        }
        Command::none()
    }

    pub fn view(&self) -> Element<GifPlayerMessage> {
        image::Image::new(self.frames[self.current_frame].clone())
            .width(44)
            .height(66)
            .into()
    }

    pub fn subscription(&self) -> Subscription<GifPlayerMessage> {
        
        iced_time::every(self.frame_delay).map(|_| GifPlayerMessage::NextFrame)
    }
}

fn load_gif_frames(path: &str) -> (Vec<image::Handle>, Duration) {
    let mut frames = Vec::new();
    let mut decoder = Decoder::new(File::open(path).unwrap()).unwrap();
    let mut frame_delay = Duration::from_secs(1);
    let palette = decoder.global_palette().unwrap().to_vec();
    let canvas_width = decoder.width() as usize;
    let canvas_height = decoder.height() as usize;
    

    println!("{:?}", palette);

    while let Some(frame) = decoder.read_next_frame().unwrap() {
        let frame_width = frame.width as usize;
        let frame_height = frame.height as usize;

        // Clear the canvas (optional, depending on GIF disposal method)
        let mut canvas = vec![0; canvas_width * canvas_height * 4]; // Full canvas
        for y in 0..frame_height {
            for x in 0..frame_width {
                let frame_index = y * frame_width + x;
                let canvas_x = frame.left as usize + x;
                let canvas_y = frame.top as usize + y;
                let canvas_index = (canvas_y * canvas_width + canvas_x) * 4;

                let palette_index = frame.buffer[frame_index] as usize * 3;
                canvas[canvas_index] = palette[palette_index];       // R
                canvas[canvas_index + 1] = palette[palette_index + 1]; // G
                canvas[canvas_index + 2] = palette[palette_index + 2]; // B
                canvas[canvas_index + 3] = 255; // Fully opaque
            }
        }
        // let mut buffer = vec![0; canvas_width * canvas_height * 4 ];
        // //println!("{:?}", frame.buffer);
        
        // let mut i = 0;

        // for &index in frame.buffer.iter() {
        //     let p_index = index as usize * 3;
        //     buffer[i] = palette[p_index];
        //     buffer[i+1] = palette[p_index + 1];
        //     buffer[i+2] = palette[p_index + 2];
        //     buffer[i+3] = 255;
        //     i += 4;
        // }

        // println!("{} {}", canvas_width, canvas_height);       
        let handle = image::Handle::from_pixels(canvas_width as u32, canvas_height as u32, canvas);
        frames.push(handle);
        frame_delay = Duration::from_millis((frame.delay as u64) * 10); // Convert delay to milliseconds
    }

    (frames, frame_delay)
}
