#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod utils_ffmpeg;
mod workers;

use eframe::egui;
use eframe::egui::{ColorImage, TextureHandle, Image};

use crate::utils_ffmpeg::check_ffmpeg;
use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent::OutputFrame, event::OutputVideoFrame, child::FfmpegChild};

use std::net::UdpSocket;
use local_ip_address::local_ip;

use std::thread;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{self, Receiver, Sender};
use crossbeam_channel::{bounded, Sender as CrossbeamSender, Receiver as CrossbeamReceiver};

use std::process::{ChildStdin};
use std::io::{Read, Write, BufReader};
use std::io::ErrorKind;

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

use rand::Rng;
use std::time::Duration;

const BUFFER_SIZE: usize = 1024;
struct MyApp {
    texture: Option<TextureHandle>,
    frames: Receiver<ColorImage>, // Condividi i frames tra thread
    is_recording: Arc<Mutex<bool>>,
    rx_record: CrossbeamReceiver<Vec<u8>>,
    pid_record: Option<Pid>,
}

impl MyApp {
    fn new(frames: Receiver<ColorImage>, is_recording: Arc<Mutex<bool>>, rx_record: CrossbeamReceiver<Vec<u8>>) -> Self {
        Self {
            texture: None,
            frames,
            is_recording,
            rx_record,
            pid_record: None,
        }
    }

    // Aggiorna immagine a 30 FPS
    fn update_image(&mut self, ctx: &egui::Context) {
        if let Ok(image) = self.frames.try_recv() {
            self.texture = Some(ctx.load_texture("updated_image", image, Default::default()));
        }
    }

    fn start_record(stdin: Mutex<ChildStdin>, rx_record: CrossbeamReceiver<Vec<u8>>){
        let mut stdin = stdin.lock().unwrap();
        loop {
            match rx_record.recv() {
                Ok(data) => {
                    match stdin.write_all(&data){
                        Ok(_) => {},
                        Err(e) if e.kind() == ErrorKind::BrokenPipe => {
                            eprintln!("Closed record process: {}", e);
                            break;
                        },
                        Err(e) => {
                            eprintln!("Failed to write data to record: {}", e);
                            break;
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Failed to receive data record: {}", err);
                }
            }
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
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                if let Some(pid) = self.pid_record {
                        if ui.add_sized([100.0, 40.0], egui::Button::new("Stop Record")).clicked() {
                            let mut recording_guard = self.is_recording.lock().unwrap();
                            if *recording_guard && self.pid_record.is_some(){
                                match signal::kill(pid, Signal::SIGINT){
                                    Ok(_) => {
                                        println!("Record process killed");
                                        self.pid_record = None;
                                        *recording_guard = false;
                                    },
                                    Err(e) => {
                                        eprintln!("Failed to kill record process: {}", e);
                                    }
                                }
                            }
                            drop(recording_guard);
                        }
                }else{
                    if ui.add_sized([100.0, 40.0], egui::Button::new("Start Record")).clicked() {
                        let mut recording_guard = self.is_recording.lock().unwrap();
                        if !*recording_guard && self.pid_record.is_none() {
                            // Configura ffmpeg-sidecar per registrare
                            let mut ffmpeg_command_record = FfmpegCommand::new()
                                .input("pipe:0")
                                .args(&["-c:v", "copy"])
                                .output("output.mp4")
                                .spawn()
                                .expect("Impossibile avviare ffmpeg per registrare");
                            let mut stderr_record = ffmpeg_command_record.take_stderr().unwrap();
                            thread::spawn(move || {
                                let mut buffer = [0; 256];
                                loop {
                                    let n = stderr_record.read(&mut buffer).unwrap();
                                    if n == 0 {
                                        break;
                                    }
                                    eprintln!("Record Process: {}", String::from_utf8_lossy(&buffer[..n]));
                                }
                            });

                            let stdin_mutex = Mutex::new(ffmpeg_command_record.take_stdin().unwrap());
                            self.pid_record = Some(Pid::from_raw(ffmpeg_command_record.as_inner_mut().id() as i32));
                            let rx_record_clone = self.rx_record.clone();
                            thread::spawn( || {
                                MyApp::start_record(stdin_mutex, rx_record_clone);
                            });
                            *recording_guard = true;
                            drop(recording_guard);
                        }
                    }
                }    
            }); 
        });
        ctx.request_repaint_after(Duration::from_secs_f32(1.0 / 30.0));
    }
}


fn main() {
    //Check ffmpeg
    check_ffmpeg().expect("Errore nel controllo di FFmpeg");

    //Source address
    let destination_ip = "192.168.1.35"; //defined manually by the user
    let target_address = format!("{destination_ip}:8080");

    //Check and get local ip address
    let ip_address: String;
    match local_ip() {
        Ok(ip) => ip_address = ip.to_string(),
        Error=> {
            println!("Impossibile ottenere l'indirizzo IP");
            panic!()
        },
    };

    //Define socket
    let socket = UdpSocket::bind(format!("{ip_address}:3040")).expect("Failed to bind socket");  // Il client bind sulla porta 8080

    let mut buffer = [0; BUFFER_SIZE];
    let message = "START".as_bytes();
    socket.set_read_timeout(Some(Duration::from_secs(10))).expect("Failed to set read timeout");

    loop{
        socket.send_to(&message, &target_address).expect("Failed to send START message");
        match socket.recv(&mut buffer) {
            Ok(number_of_bytes) => {
                let data = &buffer[..number_of_bytes];
                if data == "OK".as_bytes() {
                    buffer = [0; BUFFER_SIZE];
                    break;
                }
            }
            Err(err) => {
                eprintln!("Failed to receive data: {}", err);
            }
        }
    }

    //Define playback channels
    let (sender_image, receiver_image): (Sender<ColorImage>, Receiver<ColorImage>) = mpsc::channel();
    let (sender_frame, receiver_frame): (Sender<OutputVideoFrame>, Receiver<OutputVideoFrame>) = mpsc::channel();

    //Define channels and buffer to manage socket
    let (tx_playback, rx_playback): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();
    let (tx_record, rx_record): (CrossbeamSender<Vec<u8>>, CrossbeamReceiver<Vec<u8>>) = bounded(10); //rx_record to be stored in the appstate

    //Define recording flag
    let is_recording = Arc::new(Mutex::new(false));
    let is_recording1 = is_recording.clone();
    
    // SOCKET MANAGER
    thread::spawn(move || {
        loop {
            match socket.recv(&mut buffer) {
                Ok(number_of_bytes) => {
                    let data = &buffer[..number_of_bytes];
                    if let Err(err) = tx_playback.send(data.to_vec()) {
                        eprintln!("Failed to send data to playback: {}", err);
                    }
                    let is_recording_guard = is_recording1.lock().unwrap();
                    if *is_recording_guard {
                        drop(is_recording_guard);
                        if let Err(err) = tx_record.send(data.to_vec()) {
                            eprintln!("Failed to send data to record: {}", err);
                        }
                    }else{
                        drop(is_recording_guard);
                    }
                    
                }
                Err(err) => {
                    eprintln!("Failed to receive data: {}", err);
                    break;
                }
            }
        }
    });

    // PLAYBACK
    thread::spawn(move || {
        // Configura ffmpeg-sidecar per ricevere dati tramite UDP
        let mut ffmpeg_command = FfmpegCommand::new()
                //.input("udp:/192.168.1.95:1936?overrun_nonfatal=1&fifo_size=50000000")
                .input("pipe:0")
                .args(&["-fflags", "nobuffer", "-flags", "low_delay", "-vf", "scale=1280:720"])
                .rawvideo()
                .spawn()
                .expect("Impossibile avviare ffmpeg");
        let mut stdin = ffmpeg_command.take_stdin().unwrap();
        //DECODE AND PLAY
        let w_manager = Arc::new(workers::WorkersManger::new(5, Arc::new(Mutex::new(receiver_frame)), sender_image));
        let w_manager2 = w_manager.clone();
        thread::spawn(move || {
                // Itera sugli eventi di output di ffmpeg
                ffmpeg_command.iter().expect("Errore iterando i frame").for_each(|e| {
                    match e {
                        OutputFrame(frame) => sender_frame.send(frame).unwrap(),                
                        _ => {},//println!("Event: {:?}", e),
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

        loop {
            match rx_playback.recv() {
                Ok(data) => {
                    stdin.write_all(&data).unwrap();
                }
                Err(err) => {
                    eprintln!("Failed to receive data playback: {}", err);
                }
            }
        }
    });
    // Configura la GUI
    let options = eframe::NativeOptions {
        vsync: true,
        ..Default::default()
    };
    let _ = eframe::run_native("Image Viewer 30 FPS", options, Box::new(|_cc| Ok(Box::new(MyApp::new(receiver_image, is_recording, rx_record)))));
   }