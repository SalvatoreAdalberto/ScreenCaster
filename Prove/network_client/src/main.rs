#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod utils_ffmpeg;

use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};
use eframe::egui;
use eframe::egui::{ColorImage, TextureHandle};
use ffmpeg_sidecar::{command::FfmpegCommand};
use std::thread;
use crate::utils_ffmpeg::check_ffmpeg;

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
                ui.image(texture.id(), ui.available_size());
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
    let (sender, receiver): (Sender<ColorImage>, Receiver<ColorImage>) = mpsc::channel();

    // Creare thread separato per ricevere i frame da ffmpeg
    let sender_clone = sender.clone();

    thread::spawn(move || {
        // Configura ffmpeg-sidecar per ricevere dati tramite UDP
        let mut ffmpeg_command = FfmpegCommand::new()
            .input("udp://127.0.0.1:1235")
            .rawvideo()
            .spawn()
            .expect("Impossibile avviare ffmpeg");

        // Itera sugli eventi di output di ffmpeg
        ffmpeg_command.iter().expect("Errore iterando i frame").filter_frames().for_each(|e| {
            // Convertire i dati del frame in ColorImage
            let width = e.width as usize;
            let height = e.height as usize;

            let image_data = ColorImage::from_rgb([width, height], &e.data);
            sender_clone.send(image_data).expect("Errore nell'invio del frame");
        });
    });

    // Configura la GUI
    let options = eframe::NativeOptions {
        vsync: true,
        ..Default::default()
    };
    eframe::run_native("Image Viewer 30 FPS", options, Box::new(|_cc| Box::new(MyApp::new(receiver))));
}