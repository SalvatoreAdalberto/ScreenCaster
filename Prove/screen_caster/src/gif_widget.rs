use iced::{
    time as iced_time, widget::image::Handle, widget::image::Image, Command, Element, Subscription
};


use gif::{Decoder, Frame};
use std::fs::File;
use std::error::Error;
use std::time::Duration;
use image;

pub struct GifPlayer {
    frames: Vec<Handle>, // Store GIF frames as images
    current_frame: usize,
    delays: Vec<Duration>,
    frame_delay: Duration,
}

#[derive(Debug, Clone)]
pub enum GifPlayerMessage {
    NextFrame,
}

impl GifPlayer {
    
    pub fn new() -> Self {
        // Load GIF frames
        let (frames, delays) = load_gif("../assets/spinner1.gif").unwrap();
        let frame_delay = delays[0];
        
            GifPlayer {
                frames,
                current_frame: 0,
                delays,
                frame_delay,
            }
    
    }

    pub fn update(&mut self, message: GifPlayerMessage) -> Command<GifPlayerMessage> {
        match message {
            GifPlayerMessage::NextFrame => {
                self.current_frame = (self.current_frame + 1) % self.frames.len();

                // Imposta il ritardo per il prossimo frame
                self.frame_delay = self.delays[self.current_frame];
        }
            }

        Command::none()
    }

    pub fn view(&self) -> Element<GifPlayerMessage> {
        Image::new(self.frames[self.current_frame].clone())
            .width(150)
            .height(150)
            .into()
    }

    pub fn subscription(&self) -> Subscription<GifPlayerMessage> {
        
        iced_time::every(self.frame_delay).map(|_| GifPlayerMessage::NextFrame)
    }

}


     // Carica una GIF e converte i frame in handle di immagini Iced
    fn load_gif(path: &str) -> Result<(Vec<Handle>, Vec<Duration>), Box<dyn Error>> {
        let file = std::fs::File::open(path)?;
        let mut gif_opts = gif::DecodeOptions::new();
        
        gif_opts.set_color_output(gif::ColorOutput::Indexed);

        let mut decoder = gif_opts.read_info(file)?;
        let mut screen = gif_dispose::Screen::new_decoder(&decoder);

        let mut frames: Vec<Handle> = Vec::new();
        let mut delays: Vec<Duration> = Vec::new();

        while let Some(frame) = decoder.read_next_frame()? {
            screen.blit_frame(&frame)?;
            let (buf, width, height) = screen.pixels_rgba().to_contiguous_buf();
            let img = gif_frame_to_rgba8(buf.into_owned(), width, height).unwrap();
            frames.push(img);
            // Aggiungi il delay del frame (in millisecondi)
            let delay = Duration::from_millis(frame.delay as u64 * 10 );
            delays.push(delay);
        }

        Ok((frames, delays))
    }

// Funzione per convertire un frame `gif::Frame` in un `RgbaImage`
pub fn gif_frame_to_rgba8(buf: Vec<rgb::RGBA8>, width: usize, height: usize) -> Result<Handle, Box<dyn Error>> {
    // Crea una RgbaImage (un buffer immagine RGBA)
    let mut img = image::RgbaImage::new(width as u32, height as u32);
    let mut x = 0;
    let mut y = 0;
    let mut rgba_pixel;
    // Copia i dati del frame nella RgbaImage
    for rgba in buf.iter() {
       
        rgba_pixel = image::Rgba([rgba.r, rgba.g, rgba.b, rgba.a]);
        
        img.put_pixel(x as u32, y as u32, rgba_pixel);
        x += 1;
        if x == width {
            y += 1;
            if y == height{
                break;
            }
            x = 0;
        }
        

    }
    let img_handle = Handle::from_pixels(width as u32, height as u32, img.to_vec());
    Ok(img_handle)
    }
