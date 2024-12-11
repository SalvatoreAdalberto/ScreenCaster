#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent, event::OutputVideoFrame};

use std::net::UdpSocket;
use local_ip_address::local_ip;

use std::sync::{Arc, Mutex, atomic::AtomicBool, atomic::Ordering};
use std::{io, thread};
use std::io::{Error};
use std::sync::mpsc::{self, Receiver, Sender};
use crossbeam_channel::{bounded, Sender as CrossbeamSender, Receiver as CrossbeamReceiver};

use std::process::{ChildStdin};
use std::io::{Read, Write, BufWriter};
use std::io::ErrorKind;
use chrono::Local;
use std::time::{Instant, Duration};
use crate::workers::FrameProcessorConstructor;
use crate::gif_widget::{GifPlayer, GifPlayerMessage};

use iced::{ Subscription, time as iced_time, Command, Element, Length};
use iced::widget::{Button, image::Handle, image::Image, Text};

const LOADING_IMG: &str = "—Pngtree—blue circular progress bar page_6476398.png";
const BUFFER_SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub enum VideoPlayerMessage {
    Connect,
    NextFrame,
    Exit,
    StartRecord,
    StopRecord,
    StreamAvailable,
    NoStreamAvailable,
    NoConnection,
    GifPlayerMessage(GifPlayerMessage),
}

pub enum StreamingClientStateEnum{
    NotConnected,
    ConnectedNoStreaming,
    Streaming,
    Retry,
}

pub struct StreamingClient {
    receiver_image: Option<Receiver<Handle>>,
    is_recording: Option<Arc<Mutex<bool>>>,
    rx_record: Option<CrossbeamReceiver<Vec<u8>>>,
    pid_record: Option<i32>,
    stdin_record: Option<Arc<Mutex<ChildStdin>>>,
    target_address: String,
    own_ip: String,
    current_frame: Handle,
    socket: Arc<UdpSocket>,
    tx_connection_status: Option<Sender<VideoPlayerMessage>>,
    rx_connection_status: Option<Receiver<VideoPlayerMessage>>,
    gif_widget: Option<GifPlayer>,
    state: StreamingClientStateEnum,
}

impl StreamingClient {
    pub fn new(source_ip: String) -> Self {
        let target_address = format!("{source_ip}:8080");
        //Check and get local ip address
        let ip_address: String;
        match local_ip() {
            Ok(ip) => ip_address = ip.to_string(),
            Err(e)=> {
                println!("Impossibile ottenere l'indirizzo IP. Errore: {}", e);
                panic!()
            },
        };

        //Define socket
        let socket = Arc::new(UdpSocket::bind(format!("{ip_address}:3040")).expect("Failed to bind socket"));  // Il client bind sulla porta 8080
        let current_frame = Handle::from_path(LOADING_IMG);
       
        Self {
            current_frame,
            receiver_image: None,
            is_recording: None,
            rx_record: None,
            pid_record: None,
            stdin_record: None,
            target_address,
            own_ip: ip_address,
            socket,
            tx_connection_status: None,
            rx_connection_status: None,
            gif_widget: Some(GifPlayer::new()),
            state: StreamingClientStateEnum::NotConnected,
        }
    }

    fn update_image(&mut self) -> Option<Handle>{
        if let Ok(image) = self.receiver_image.as_ref().unwrap().try_recv() {
            Some(image)
        }
        else {
            None
        }
    }

    fn start_connection(&mut self) -> (Sender<VideoPlayerMessage>, Receiver<VideoPlayerMessage>) {

        let mut buffer = [0; BUFFER_SIZE];
        let message = "START".as_bytes();
        let target = self.target_address.clone();
        let (tx_connection_status, rx_connection_status) = mpsc::channel();
        let socket_clone = self.socket.clone();
        socket_clone.set_read_timeout(Some(Duration::from_secs(1))).expect("Failed to set read timeout");
        let start = Instant::now();
        let tx_sc = tx_connection_status.clone();
        // INIT CONNECTION
        thread::spawn(move||{
            loop {
                if start.elapsed() > Duration::from_secs(10) {
                    eprintln!("Connection timeout");
                    tx_sc.send(VideoPlayerMessage::NoConnection).unwrap();
                    break;
                }
                match socket_clone.send_to(&message, &target) {
                    Ok(_) => {
                        match socket_clone.recv(&mut buffer) {
                            Ok(number_of_bytes) => {
                                let data = &buffer[..number_of_bytes];
                                if data == "OK".as_bytes() {
                                    tx_sc.send(VideoPlayerMessage::NoStreamAvailable).unwrap();
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }); 

       (tx_connection_status, rx_connection_status)
    }
    
    fn manage_incoming_packets(&mut self){
        let mut buffer = [0; BUFFER_SIZE];
        //Define playback channels
        let (sender_image, receiver_image): (Sender<Handle>, Receiver<Handle>) = mpsc::channel();
        let (sender_frame, receiver_frame): (Sender<OutputVideoFrame>, Receiver<OutputVideoFrame>) = mpsc::channel();

        //Define channels and buffer to manage socket
        let (tx_playback, rx_playback): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();
        let (tx_record, rx_record): (CrossbeamSender<Vec<u8>>, CrossbeamReceiver<Vec<u8>>) = bounded(10); //rx_record to be stored in the appstate

        //Define recording flag
        let is_recording = Arc::new(Mutex::new(false));
        let is_recording1 = is_recording.clone();

        //Clone socket
        let socket_clone = self.socket.clone();

        self.receiver_image = Some(receiver_image);
        self.rx_record = Some(rx_record);
        self.is_recording = Some(is_recording);
        let stop_receiving = Arc::new(AtomicBool::new(false));
        let stop_receiving_ffpmeg = stop_receiving.clone();

        let tx_sm = self.tx_connection_status.as_ref().unwrap().clone();
        let tx_pb = self.tx_connection_status.as_ref().unwrap().clone();

        // SOCKET MANAGER
        thread::spawn(move || {
            loop {
                match socket_clone.recv(&mut buffer) {
                    Ok(number_of_bytes) => {
                        let data = &buffer[..number_of_bytes];
                        if let Err(err) = tx_playback.send(data.to_vec()) {
                            eprintln!("Failed to send data to playback: {}", err);
                            break;
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
                    Err(_) => {
                        tx_sm.send(VideoPlayerMessage::NoConnection);
                        break;
                    }
                }
            }
            println!("Ending thread-socketManager-1");
        });
        // PLAYBACK
        thread::spawn(move || {
            // Configura ffmpeg-sidecar per ricevere dati tramite UDP
            let mut ffmpeg_command = FfmpegCommand::new()
                    //.input("udp:/192.168.1.95:1936?overrun_nonfatal=1&fifo_size=50000000")
                    .input("pipe:0")
                    .args(&[ "-fflags", "nobuffer", "-flags", "low_delay", "-vf", "scale=1280:720",])
                    .rawvideo()
                    .spawn()
                    .expect("Impossibile avviare ffmpeg");
            let mut stdin = ffmpeg_command.take_stdin().unwrap();
            let mut writer = BufWriter::new(&mut stdin);
            //DECODE AND PLAY
            let  ( dispatcher, mut aggregator )= FrameProcessorConstructor::new(5, receiver_frame, sender_image);
            thread::spawn(move || {
                    // Itera sugli eventi di output di ffmpeg
                    for e in ffmpeg_command.iter().expect("Errore iterando i frame"){
                        match e {
                            FfmpegEvent::OutputFrame(frame) => {
                                match sender_frame.send(frame){
                                    Ok(_) => {},
                                    Err(_) => {break},
                                }
                            },     
                            FfmpegEvent::ParsedOutputStream(_) => {
                                tx_pb.send(VideoPlayerMessage::StreamAvailable).unwrap();
                            }           
                            _ => {},
                        }
                    };


                    drop(sender_frame);
                    stop_receiving.store(true, Ordering::Relaxed);
                    println!("Ending thread-externalSendFrame-2");

                });
            thread::spawn(move || {
                dispatcher.execute();
                println!("Ending thread-executeManager-3");
            });
            thread::spawn(move ||{
                aggregator.activate();
                aggregator.join_workers();
                println!("Ending thread-activateManager-4");
            });
            
            while !stop_receiving_ffpmeg.load(Ordering::Relaxed) {
                match rx_playback.recv() {
                    Ok(data) => {
                        writer.write_all(&data).unwrap();
                        
                    }
                  Err(err) => {
                        //eprintln!("Failed to receive data playback: {}", err);
                        
                        break;
                    }
                }
            }
            drop(rx_playback);
            writer.write_all(b"").unwrap();

            println!("Ending thread-ffmpegStdinWriter-5");
           });
    }

    fn on_exit(&mut self) {
        let socket = Arc::new(UdpSocket::bind(format!("{}:3043", self.own_ip)).expect("Failed to bind socket"));  
        let mut buffer = [0; BUFFER_SIZE];
        let message = format!("STOP\n{}:3040", self.own_ip);
        socket.set_read_timeout(Some(Duration::from_secs_f32(0.2))).expect("Failed to set read timeout");
        let start = Instant::now();
        println!("Asking to stop connection");
        loop{
            if start.elapsed() > Duration::from_secs(1) {
                eprintln!("Connection timeout");
                break;
            }
            socket.send_to(&message.as_bytes(), &self.target_address);
            match socket.recv(&mut buffer) {
                Ok(number_of_bytes) => {
                    let data = &buffer[..number_of_bytes];
                    if data == "OK".as_bytes() {
                        break;
                    }
                }
                Err(err) => {
                    eprintln!("Failed to receive data: {}", err);
                }
            }
        }
        drop(socket);
    }

    fn start_record(&mut self) {
        let mut recording_guard = self.is_recording.as_ref().unwrap().lock().unwrap();
            if !*recording_guard && self.pid_record.is_none() {
                let file_name = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
                // Configura ffmpeg-sidecar per registrare
                let mut ffmpeg_command_record = FfmpegCommand::new()
                    .input("pipe:0")
                    .args(&["-fflags","discardcorrupt","-c:v", "copy", "-y"])
                    .output(format!("{file_name}.mp4"))
                    .spawn()
                    .expect("Impossibile avviare ffmpeg per registrare");
                
                let stdin_mutex = Arc::new(Mutex::new(ffmpeg_command_record.take_stdin().unwrap()));
                let stdin_mutex_clone = stdin_mutex.clone();
                let mut stderr = ffmpeg_command_record.take_stderr().unwrap();
                thread::spawn(move || {
                    let mut buffer = [0;256];

                    loop{
                        let n = stderr.read(&mut buffer).unwrap();
                        if n == 0{
                            break;
                        }
                        eprintln!("Record Process: {}", String::from_utf8_lossy(&buffer[..n]));
                    }

                });
                self.pid_record = Some(ffmpeg_command_record.as_inner().id() as i32);
                let rx_record_clone = self.rx_record.as_ref().unwrap().clone();
                thread::spawn( || {
                    StreamingClient::feed_record_raw(stdin_mutex_clone, rx_record_clone);
                });
                self.stdin_record = Some(stdin_mutex);
                *recording_guard = true;
                drop(recording_guard);
            }
    }

    fn feed_record_raw(stdin: Arc<Mutex<ChildStdin>>, rx_record: CrossbeamReceiver<Vec<u8>>){
        loop {
            match rx_record.recv_timeout(Duration::from_secs(1)) {
                Ok(data) => {
                    let mut stdin = stdin.lock().unwrap();
                    let mut writer = BufWriter::new(&mut *stdin);
                    match writer.write_all(&data){
                        Ok(_) => {
                        },
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
                    break;
                }
            }
        }
    }

    fn stop_record(&mut self){
        let mut recording_guard = self.is_recording.as_ref().unwrap().lock().unwrap();
        if *recording_guard && self.pid_record.is_some(){
            let mut stdin_record = self.stdin_record.as_mut().unwrap().lock().unwrap();
            stdin_record.flush().unwrap();
            match stdin_record.write_all(b""){
                Ok(_) => {
                    println!("Record process killed");
                    drop(stdin_record);
                    self.pid_record = None;
                    self.stdin_record = None;   
                    *recording_guard = false;
                },
                Err(e) => {
                    eprintln!("Failed to kill record process: {}", e);
                }
            }
        }
        drop(recording_guard);
    }

    pub fn update(&mut self, message: VideoPlayerMessage) -> Option<VideoPlayerMessage> {
        let mut tmp_message = message.clone();
        if self.rx_connection_status.is_some(){
            if let Ok(inner_message) = self.rx_connection_status.as_mut().unwrap().try_recv(){
               match message{
                            VideoPlayerMessage::GifPlayerMessage(_) | VideoPlayerMessage::NextFrame=> {
                               tmp_message = inner_message;
                            },
                            _ => {}
                        }
                    }
                }
   
            return match tmp_message {
                VideoPlayerMessage::Connect => {
                    self.state = StreamingClientStateEnum::NotConnected;
                    let (tx, rx) = self.start_connection();
                    self.tx_connection_status = Some(tx);
                    self.rx_connection_status = Some(rx);
                    None
                }
                VideoPlayerMessage::NoConnection => {
                    self.state = StreamingClientStateEnum::Retry;
                    println!("HERE");
                    None
                }
                VideoPlayerMessage::NoStreamAvailable =>{
                    self.state = StreamingClientStateEnum::ConnectedNoStreaming;
                    self.manage_incoming_packets();
                    None
                }
                VideoPlayerMessage::StreamAvailable => {
                    self.state = StreamingClientStateEnum::Streaming;
                    None
                }
                VideoPlayerMessage::Exit => {
                    if let Some(_) = self.pid_record {
                        self.stop_record();
                    }
                    self.on_exit();
                    None
                }    
                VideoPlayerMessage::NextFrame => {
                    if let Some(image) = self.update_image() {
                        self.current_frame = image;
                    }
                    None
                }
                VideoPlayerMessage::GifPlayerMessage(GifPlayerMessage) => {
                    if let  Some(gif) = self.gif_widget.as_mut(){
                        gif.update(GifPlayerMessage);
                    } 
                    None
                }
                
                VideoPlayerMessage::StartRecord => {
                    self.start_record();
                    None
                }
                VideoPlayerMessage::StopRecord => {
                    self.stop_record();
                    None
                }
            }
        }
    
    pub fn view_video(&self) -> Element<VideoPlayerMessage>{
        match self.state{
            StreamingClientStateEnum::Streaming => {
                Image::new(self.current_frame.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            },
            StreamingClientStateEnum::Retry => {
                Button::new(Text::new("Riprova"))
                    .padding(10)
                    .width(Length::Fixed(200.0))
                    .on_press(VideoPlayerMessage::Connect)
                    .into()
            }
            _ => {
                    self.gif_widget.as_ref().unwrap().view().map(VideoPlayerMessage::GifPlayerMessage)
                }
        }
            
    }

    pub fn view_record_button(&self) -> Option<Element<VideoPlayerMessage>> {
        match self.state{
            StreamingClientStateEnum::Streaming => {
                if let Some(_) = self.pid_record{
                    Some(Button::new(Text::new("Stop Record"))
                    .padding(10)
                    .width(Length::Fixed(200.0))
                    .on_press(VideoPlayerMessage::StopRecord)
                    .into())
                }else{
                    Some(Button::new(Text::new("Start Record"))
                    .padding(10)
                    .width(Length::Fixed(200.0))
                    .on_press(VideoPlayerMessage::StartRecord)
                    .into())
                }
            },
            _ => {None}
        }
       
    }

    pub fn subscription(&self) -> Subscription<VideoPlayerMessage>{
            match self.state{
                StreamingClientStateEnum::Streaming => {iced_time::every(Duration::from_secs_f32(1.0/40.0 )).map(|_| VideoPlayerMessage::NextFrame)},
                _ => {
                    if let Some(gif) = self.gif_widget.as_ref(){
                        gif.subscription().map(VideoPlayerMessage::GifPlayerMessage)
                    }else{ 
                        Subscription::none()
                    }
                }
            }
            
    }
}
impl Drop for StreamingClient {
    fn drop(&mut self) {
        println!("Dropping Streaming Client");
        if let Some(_) = self.pid_record {
            self.stop_record();
        }
        self.on_exit();
    }
}


