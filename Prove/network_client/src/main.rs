#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod utils_ffmpeg;
mod workers;

<<<<<<< HEAD
use std::sync::{Arc};
=======
use std::sync::{Arc, Mutex};
>>>>>>> 5f8f66d0 (added buffer/channel between ffmpeg and workers manager on client side)
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;
use eframe::egui;
use eframe::egui::{ColorImage, TextureHandle, Image};
use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent::OutputFrame, event::OutputVideoFrame};
use std::thread;
use crate::utils_ffmpeg::check_ffmpeg;

struct Frame{
    data: Vec<u8>,
    width: u32,
    height: u32,
}

struct MyApp {
    texture: Option<TextureHandle>,
    frames: Receiver<ColorImage>, // Condividi i frames tra thread
}

impl MyApp {
    fn new(frames: Receiver<ColorImage>) -> Self {
        Self {
            texture: None,
            frames,
        }
    }

    // Aggiorna immagine a 30 FPS
    fn update_image(&mut self, ctx: &egui::Context) {
        if let Ok(image) = self.frames.try_recv() {
            self.texture = Some(ctx.load_texture("updated_image", image, Default::default()));
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_image(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.texture {
                ui.add(Image::new(texture).fit_to_exact_size(ui.available_size()));
            } else {
                ui.label("Nessuna immagine disponibile.");
            }
        });

        ctx.request_repaint_after(Duration::from_secs_f32(1.0 / 30.0));
    }
}

fn main() {
    check_ffmpeg().expect("Errore nel controllo di FFmpeg");
    // Creare canale per inviare i frame

    let (sender_image, receiver_image): (Sender<ColorImage>, Receiver<ColorImage>) = mpsc::channel();
    let (sender_frame, receiver_frame): (Sender<OutputVideoFrame>, Receiver<OutputVideoFrame>) = mpsc::channel();

    
        // Configura ffmpeg-sidecar per ricevere dati tramite UDP
    let mut ffmpeg_command = FfmpegCommand::new()
            .input("udp:/192.168.0.100:1936?overrun_nonfatal=1&fifo_size=50000000")
            .args(&["-fflags", "nobuffer", "-flags", "low_delay", "-vf", "scale=1920:1080"])
            .rawvideo()
            .spawn()
            .expect("Impossibile avviare ffmpeg");

    let w_manager = Arc::new(workers::WorkersManger::new(5, Arc::new(Mutex::new(receiver_frame)), sender_image));
    let w_manager2 = w_manager.clone();
    thread::spawn(move || {
        // Itera sugli eventi di output di ffmpeg
        ffmpeg_command.iter().expect("Errore iterando i frame").for_each(|e| {
            match e {
                OutputFrame(frame) => sender_frame.send(frame).unwrap(),                
                _ => println!("Event: {:?}", e),
            }
            //println!("len: {:?} ", e);
            
        });
    });
    thread::spawn(move || {
        w_manager2.execute();
    });
    thread::spawn(move ||{
        w_manager.activate();
    });

    // Configura la GUI
    let options = eframe::NativeOptions {
        vsync: true,
        ..Default::default()
    };
    eframe::run_native("Image Viewer 30 FPS", options, Box::new(|_cc| Ok(Box::new(MyApp::new(receiver_image)))));
}