use std::net::UdpSocket;

use std::sync::mpsc::channel;
use std::thread;
use std::sync::{Arc, Condvar, Mutex};

use local_ip_address::local_ip;

use std::collections::HashMap;
use std::fs::File;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::child::FfmpegChild;
use std::io::{Read, Write, BufReader};
use std::time::Duration;
use crate::gui::ShareMode;
use crate::utils;

/// This module contains the StreamingServer struct and its implementation.
/// The StreamingServer struct is responsible for starting and stopping the screen casting process.
/// The StreamingServer is in charge of sending the screen casting data to the clients.
/// The list of clients is updated dinamically when a new client connects or disconnects.
/// When the server is stopped, the server will notify all the connected clients and terminate the threads.

const BUFFER_SIZE: usize = 1024;

struct Client{
    tx: std::sync::mpsc::Sender<Vec<u8>>,
}

// StreamingServer struct contains the handle to the ffmpeg process, the list of connected clients, the control variable and the threads.
pub struct StreamingServer {
    handle: Option<Mutex<FfmpegChild>>,
    list_clients: Arc<Mutex<HashMap<String, Client>>>,
    control: Arc<(Mutex<bool>, Condvar)>,
    threads: Vec<thread::JoinHandle<()>>,
}

// CropArea struct contains the width, height, x_offset and y_offset of the crop area.
#[derive(Debug)]
pub struct CropArea {
    pub width: u32,
    pub height: u32,
    pub x_offset: u32,
    pub y_offset: u32,
}

impl StreamingServer {
    pub fn new() -> Self {
        StreamingServer {
            handle: None,
            list_clients: Arc::new(Mutex::new(HashMap::new())),
            control: Arc::new((Mutex::new(false), Condvar::new())), 
            threads: Vec::new(),
        }
    }

    // Start the screen casting process. Also start a thread to listen for incoming connections and a thread to send the screen casting data to the clients.
    pub fn start(&mut self, screen_index: usize, share_mode: ShareMode) {

        {
            // Reset the control variable, made up of a mutex and a condition variable
            let (lock, cvar) = &*self.control;
            let mut terminate = lock.lock().unwrap();
            *terminate = false;
            cvar.notify_all();
        }

        let command ;

        // Get the FFmpeg command to start the screen casting process based on the screen index and the share mode.
        if share_mode == ShareMode::CropArea {
            let exe_path = utils::get_project_src_path();
            let file_path;
            file_path = exe_path.display().to_string() + r"/config/crop.txt";

            let mut file = File::open(file_path).expect("Impossibile aprire il file");
            let mut content = String::new();
            file.read_to_string(&mut content).expect("Impossibile leggere il file");

            let fields: Vec<u32> = content
                .split(',')
                .map(|s| s.trim().parse::<f64>()) 
                .map(|res| res.map(|num| num.round() as u32))
                .collect::<Result<_, _>>().unwrap();

            let crop = CropArea {
                width: fields[2],
                height: fields[3],
                x_offset: fields[0],
                y_offset: fields[1],
            };
            command = utils::get_ffmpeg_command(screen_index, Some(crop));

        }
        else {
            command = utils::get_ffmpeg_command(screen_index, None);
        }

        // Get the local IP address
        let ip_address: String;
        match local_ip() {
            Ok(ip) => ip_address = ip.to_string(),
            _ => {
                println!("Impossibile ottenere l'indirizzo IP");
                panic!()
            },
        };

        // Bind the socket to the local IP address and port 8080 to listen for incoming connections
        let socket = Arc::new(UdpSocket::bind(format!("{ip_address}:8080")).expect("Failed to bind socket"));
        let listener_socket = socket.clone();

        let ffmpeg_command = command.split(" ").collect::<Vec<&str>>();

        // Start the FFmpeg process
        let mut ffmpeg = FfmpegCommand::new().args(&ffmpeg_command).spawn().expect("Failed to start FFmpeg");
        let mut reader = BufReader::new(ffmpeg.take_stdout().unwrap());
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        let handle = Mutex::new(ffmpeg);

        let control = Arc::clone(&self.control);
        let control_clone = Arc::clone(&self.control);

        // Clone the list of clients to be used in the listener thread
        let list_tx_clients_clone = Arc::clone(&self.list_clients);

        listener_socket.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

        // Start a thread to listen for incoming connections
        let h = thread::spawn(move || {
            let mut buffer = [0; BUFFER_SIZE];
            let (lock, cvar) = &*control;

            loop{

                if *lock.lock().unwrap() {
                    break;
                }

                // Receive the data from the client
                let (bytes_received, client_address) = match listener_socket.recv_from(&mut buffer) {
                    Ok(res) => res,
                    Err(_) => {
                        continue
                    }
                };

                let message = String::from_utf8_lossy(&buffer[..bytes_received]);
                let target_address = format!("{}:{}", client_address.ip(), client_address.port());

                let mut list_guard = list_tx_clients_clone.lock().unwrap();
                // If the message is "START" and the client is not in the list of clients, add the client to the list and start a thread to send the data to the client
                if message.trim() == "START"{
                    if !list_guard.contains_key(&target_address.clone()){

                        let send_socket = listener_socket.clone();
                        let (tx, rx) = channel::<Vec<u8>>();

                        list_guard.insert(target_address.clone(), Client{ tx, });

                        // Send an ACK to the client
                        listener_socket.send_to(b"OK", &target_address).unwrap();
                        
                        // Start a thread to send the data to the client
                        thread::spawn(move || {
                            loop {
                                // When the client is closed, drop the client from the list of clients
                                match rx.recv(){
                                    Ok(data) => {send_socket.send_to(&data, &target_address).unwrap();},
                                    Err(_) => {
                                        break;
                                    }
                                }
                            }
                        });
                    }else{
                        // Send an ACK to the client if the client is already in the list of clients
                        listener_socket.send_to(b"OK", &target_address).unwrap();
                    }
                }
                // If the message is "STOP" and the client is in the list of clients remove the client from the list of clients
                if message.trim().starts_with("STOP"){
                    let message = message.split("\n").collect::<Vec<&str>>();
                    let ip = message[1];
                    list_guard.remove(ip);
                    // Send an ACK to the client
                    listener_socket.send_to(b"OK", &client_address).unwrap();
                }
                drop(list_guard);
            }
            // Notify all the threads that the listener thread is terminated
            cvar.notify_all(); 
        });

        self.threads.push(h);

        // Start a thread to send the screen casting data to the clients
        let list_tx_clients_clone2 = Arc::clone(&self.list_clients);
        let h = thread::spawn(move || {
            let (lock, cvar) = &*control_clone;

            loop {
                // Check the condition variable to stop the thread
                if *lock.lock().unwrap() {
                    break;
                }

                let n = reader.read(&mut buffer).unwrap();
                // If the data is empty, break the loop
                if n == 0 {
                    break;
                }
                let clients = list_tx_clients_clone2.lock().unwrap();
                // Send the data to all the clients
                for client in clients.values() {
                    client.tx.send(buffer[..n].to_vec()).unwrap();
                }
            }

            // Notify all the threads that the sender thread is terminated
            cvar.notify_all(); 
        });

        self.threads.push(h);

        self.handle = Some(handle);

    }

    // Stop the screen casting process. Notify all the connected clients and terminate the threads.
    pub fn stop (&mut self) {
        if let Some(ref process) = self.handle {
            let mut guard = process.lock().unwrap();

            // Send "q" to the stdin of the FFmpeg process to stop the process
            if let Some(mut stdin) = (*guard).take_stdin() {
                writeln!(stdin, "q").unwrap();
            }

            guard.wait().expect("Failed to stop FFmpeg process");

            {
                // Set the condition variable to true to stop the threads
                let (lock, cvar) = &*self.control;
                let mut terminate = lock.lock().unwrap();
                *terminate = true;
                cvar.notify_all();
            }

            // Wait for the threads to terminate
            for h in self.threads.drain(..) {
                h.join().unwrap();
            }
        }
    }
}