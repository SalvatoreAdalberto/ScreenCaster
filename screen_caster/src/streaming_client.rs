use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent, event::OutputVideoFrame};

use std::net::UdpSocket;
use local_ip_address::local_ip;

use std::sync::{Arc, Mutex, atomic::AtomicBool, atomic::Ordering};
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};
use crossbeam_channel::{bounded, Sender as CrossbeamSender, Receiver as CrossbeamReceiver};

use std::process::ChildStdin;
use std::io::{ Write, BufWriter};
use std::path::PathBuf;
use chrono::Local;
use std::time::{Instant, Duration};
use crate::workers::FrameProcessorConstructor;
use crate::gif_widget::{GifPlayer, GifPlayerMessage};

use iced::{ Subscription, time as iced_time, Element, Length};
use iced::widget::{Button, image::Handle, image::Image, Text};

const BUFFER_SIZE: usize = 1024;

/// This module manages the streaming client. It is responsible for managing the connection with the server, receiving the video stream and displaying it.
/// It also manages the recording of the video stream.
/// When a new connection is issued a new StreamingClient is created.

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
    tx_connection_status: CrossbeamSender<VideoPlayerMessage>,
    rx_connection_status: CrossbeamReceiver<VideoPlayerMessage>,
    gif_widget: Option<GifPlayer>,
    state: StreamingClientStateEnum,
    save_dir: String,
}

impl StreamingClient {

    pub fn new(source_ip: String, save_dir: String) -> Self {
        let target_address = format!("{source_ip}:8080");
        //Check and get local ip address
        let ip_address: String;
        match local_ip() {
            Ok(ip) => ip_address = ip.to_string(),
            Err(_)=> {
                panic!()
            },
        };

        //Define socket
        let socket = Arc::new(UdpSocket::bind(format!("{ip_address}:3040")).expect("Failed to bind socket"));  // Il client bind sulla porta 8080
        let current_frame = Handle::from_memory([0 as u8; 1]);
        let (tx_connection_status, rx_connection_status) = bounded(1);

       
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
            tx_connection_status,
            rx_connection_status,
            gif_widget: Some(GifPlayer::new()),
            state: StreamingClientStateEnum::NotConnected,
            save_dir,
        }
    }

    /// This method receives the next frame from the the FrameProcessor 
    fn update_image(&mut self) -> Option<Handle>{
        if let Ok(image) = self.receiver_image.as_ref().unwrap().try_recv() {
            Some(image)
        }
        else {
            None
        }
    }

    /// This method initiates the connection with the server
    /// It sends a "START" message to the server and waits for a response.
    /// If the server responds with "OK" it means that we are connected but stream is not yet available.
    fn start_connection(&mut self){

        let mut buffer = [0; BUFFER_SIZE];
        let message = "START".as_bytes();
        let target = self.target_address.clone();
        let socket_clone = self.socket.clone();
        socket_clone.set_read_timeout(Some(Duration::from_secs_f32(0.5))).expect("Failed to set read timeout");
        let start = Instant::now();
        let tx_sc = self.tx_connection_status.clone();
        let rx_sc = self.rx_connection_status.clone();

        // INIT CONNECTION
        thread::spawn(move||{
            loop {
                if let Ok(VideoPlayerMessage::Exit) = rx_sc.try_recv(){
                    break;
                }
                if start.elapsed() > Duration::from_secs(5) {
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
    }
    
    /// This method manages the incoming packets from the server.
    /// It creates a new thread for the socket manager and another thread for the playback.
    /// The socket manager receives the packets from the server and sends them to the playback thread as soon as they are available.
    /// The playback thread sends the packets to ffmpeg for decoding.
    /// Then frames are sent to the FrameProcessor for processing.
    /// Socket manager also dispatches the frames to the record thread if recording is active.
    /// 
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

        let tx_sm = self.tx_connection_status.clone();
        let tx_pb = self.tx_connection_status.clone();

        // SOCKET MANAGER
        thread::spawn(move || {
            loop {
                match socket_clone.recv(&mut buffer) {
                    Ok(number_of_bytes) => {
                        let data = &buffer[..number_of_bytes];
                        if let Err(_) = tx_playback.send(data.to_vec()) {
                            break;
                        }
                        let is_recording_guard = is_recording1.lock().unwrap();
                        if *is_recording_guard {
                            drop(is_recording_guard);
                            let _  = tx_record.send(data.to_vec());
                        }else{
                            drop(is_recording_guard);
                        }

                    }
                    Err(_) => {
                        let _ = tx_sm.send(VideoPlayerMessage::NoConnection);
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

            });
            thread::spawn(move || {
                dispatcher.execute();
            });
            thread::spawn(move ||{
                aggregator.activate();
                aggregator.join_workers();
            });

            while !stop_receiving_ffpmeg.load(Ordering::Relaxed) {
                match rx_playback.recv() {
                    Ok(data) => {
                        writer.write_all(&data).unwrap();

                    }
                    Err(_) => {
                        break;
                    }
                }
            }
            drop(rx_playback);
            writer.write_all(b"").unwrap();
        });
    }

    /// This method sends a "STOP" message to the server to inform the server we are leaving.
    fn on_exit(&mut self) {
        match UdpSocket::bind(format!("{}:3043", self.own_ip)){
            Ok(s) => {
                let socket = Arc::new(s);

                let mut buffer = [0; BUFFER_SIZE];
                let address = self.target_address.clone();
                let message = format!("STOP\n{}:3040", self.own_ip);
                socket.set_read_timeout(Some(Duration::from_secs_f32(0.2))).expect("Failed to set read timeout");
                let start = Instant::now();

                loop{
                    if start.elapsed() > Duration::from_secs(1) {
                        break;
                    }
                    let _ = socket.send_to(&message.as_bytes(), &address);
                    match socket.recv(&mut buffer) {
                        Ok(number_of_bytes) => {
                            let data = &buffer[..number_of_bytes];
                            if data == "OK".as_bytes() {
                                break;
                            }
                        }
                        Err(_) => {}
                    }
                }
                drop(socket);
            },
            Err(_) => {
            }
        }
    }

    /// This method starts the recording of the video stream issuing a new ffmpeg command.
    fn start_record(&mut self) {
        let mut recording_guard = self.is_recording.as_ref().unwrap().lock().unwrap();
        if !*recording_guard && self.pid_record.is_none() {
            let file_name = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

            let save_dir = self.save_dir.clone(); // Supponendo che self.save_dir sia una String

            // Costruisci il percorso completo
            let mut file_path = PathBuf::from(save_dir);
            file_path.push(format!("{file_name}.mp4"));
            // Configura ffmpeg-sidecar per registrare
            let mut ffmpeg_command_record = FfmpegCommand::new()
                .input("pipe:0")
                .args(&["-fflags","discardcorrupt","-c:v", "copy", "-y"])
                .output(file_path.to_str().unwrap())
                .spawn()
                .expect("Impossibile avviare ffmpeg per registrare");

            let stdin_mutex = Arc::new(Mutex::new(ffmpeg_command_record.take_stdin().unwrap()));
            let stdin_mutex_clone = stdin_mutex.clone();
            
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

    /// This method feeds the record process with the incoming packets received by the socket manager thread.
    fn feed_record_raw(stdin: Arc<Mutex<ChildStdin>>, rx_record: CrossbeamReceiver<Vec<u8>>){
        loop {
            match rx_record.recv_timeout(Duration::from_secs(1)) {
                Ok(data) => {
                    let mut stdin = stdin.lock().unwrap();
                    let mut writer = BufWriter::new(&mut *stdin);
                    match writer.write_all(&data){
                        Ok(_) => {
                        },
                        Err(_) => {
                            break;
                        }
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }
    }

    /// This method stops the recording of the video stream.
    fn stop_record(&mut self){
        let mut recording_guard = self.is_recording.as_ref().unwrap().lock().unwrap();
        if *recording_guard && self.pid_record.is_some(){
            let mut stdin_record = self.stdin_record.as_mut().unwrap().lock().unwrap();
            stdin_record.flush().unwrap();
            match stdin_record.write_all(b""){
                Ok(_) => {
                    drop(stdin_record);
                    self.pid_record = None;
                    self.stdin_record = None;
                    *recording_guard = false;
                },
                Err(_) => {
                }
            }
        }
        drop(recording_guard);
    }

    pub fn update(&mut self, message: VideoPlayerMessage) -> Option<VideoPlayerMessage> {
        let mut tmp_message = message.clone();
        if let Ok(inner_message) = self.rx_connection_status.try_recv(){
            match message{
                VideoPlayerMessage::GifPlayerMessage(_) | VideoPlayerMessage::NextFrame=> {
                    tmp_message = inner_message;
                },
                _ => {}
            }
        }
        return match tmp_message {
            VideoPlayerMessage::Connect => {
                self.state = StreamingClientStateEnum::NotConnected;
                self.start_connection();
                None
            }
            VideoPlayerMessage::NoConnection => {
                self.state = StreamingClientStateEnum::Retry;
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
            // This message is used to cleanup the resources and close the connection
            VideoPlayerMessage::Exit => {
                let _ = self.tx_connection_status.send(VideoPlayerMessage::Exit);
                if let Some(_) = self.pid_record {
                    self.stop_record();
                }
                self.on_exit();
                None
            }
            // This message is used to update the image displayed on the screen, it is sent by the subscription
            VideoPlayerMessage::NextFrame => {
                if let Some(image) = self.update_image() {
                    self.current_frame = image;
                }
                None
            }
            // This message is used to manage the gif widget
            VideoPlayerMessage::GifPlayerMessage(gif_player_message) => {
                if let  Some(gif) = self.gif_widget.as_mut(){
                    let _ = gif.update(gif_player_message);
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

    /// This method returns the view of the video player
    pub fn view_video(&self) -> Element<VideoPlayerMessage>{
        match self.state{
            StreamingClientStateEnum::Streaming => {
                Image::new(self.current_frame.clone())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            },
            StreamingClientStateEnum::Retry => {
                Button::new(Text::new("Riprova").horizontal_alignment(iced::alignment::Horizontal::Center))
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
                    Some(Button::new(Text::new("Stop Record").horizontal_alignment(iced::alignment::Horizontal::Center))
                        .padding(10)
                        .width(Length::Fixed(200.0))
                        .on_press(VideoPlayerMessage::StopRecord)
                        .into())
                }else{
                    Some(Button::new(Text::new("Start Record").horizontal_alignment(iced::alignment::Horizontal::Center))
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
        let _ = self.tx_connection_status.try_send(VideoPlayerMessage::Exit);
        if let Some(_) = self.pid_record {
            self.stop_record();
        }
        self.on_exit();
    }
}