use std::net::UdpSocket;

use std::sync::mpsc::channel;
use std::thread;
use std::sync::{Arc, Mutex};

use local_ip_address::local_ip;

use std::collections::HashMap;
use std::fs::File;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::named_pipes::NamedPipe;
use ffmpeg_sidecar::pipe_name;
use ffmpeg_sidecar::event::{FfmpegEvent, LogLevel};
use std::io::{Read, Write, BufReader};
use crate::gui::ShareMode;
use crate::utils;
use anyhow;

const SHARE_PIPE_NAME: &str = pipe_name!("share_pipe");
const BUFFER_SIZE: usize = 1024;
struct Client{
    tx: std::sync::mpsc::Sender<Vec<u8>>,
}

pub struct StreamingServer {
    ready_to_share: bool,
    handle: Option<Mutex<FfmpegChild>>,
    list_clients: Arc<Mutex<HashMap<String, Client>>>,
}

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
            ready_to_share: false,
            handle: None,   
            list_clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start(&mut self, screen_index: usize, share_mode: ShareMode) -> anyhow::Result<()> {

        let mut command = "".to_string();

        if share_mode == ShareMode::CropArea {
            let exe_path = utils::get_project_src_path();
            let mut file_path = "".to_string();
            file_path = exe_path.display().to_string() + r"/config/crop.txt";

            let mut file = File::open(file_path).expect("Impossibile aprire il file");
            let mut content = String::new();
            file.read_to_string(&mut content).expect("Impossibile leggere il file");

            let fields: Vec<u32> = content
                .split(',')
                .map(|s| s.trim().parse::<f64>()) // Parse each field as f64
                .map(|res| res.map(|num| num.round() as u32)) // Round to nearest integer and convert to u32
                .collect::<Result<_, _>>().unwrap(); // Collect into Vec<u32>, propagating any errors
            println!("{:?}", fields);

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

        println!("{:?}", command);

        let ip_address: String;
        match local_ip() {
            Ok(ip) => ip_address = ip.to_string(),
            _ => {
                println!("Impossibile ottenere l'indirizzo IP");
                panic!()
            },
        };

        let socket = Arc::new(UdpSocket::bind(format!("{ip_address}:8080")).expect("Failed to bind socket"));  // Il client bind sulla porta 8080
        let listener_socket = socket.clone();

        //let mut pipe = NamedPipe::new(SHARE_PIPE_NAME)?;
        let (ready_sender, ready_receiver) = channel::<()>();
        let list_tx_clients_clone = Arc::clone(&self.list_clients);
        println!("here");

        let ffmpeg_command = command.split(" ").collect::<Vec<&str>>();
        let mut ffmpeg = FfmpegCommand::new().args(&ffmpeg_command).overwrite().output("-")
        .spawn()?;
        let mut reader = BufReader::new(ffmpeg.take_stdout().unwrap());

        thread::spawn(move || -> anyhow::Result<()>{
            ready_receiver.recv().unwrap();
            let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
            loop {
                match reader.read(&mut buffer) {
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            break;
                        }
                        let clients = list_tx_clients_clone.lock().unwrap();
                        for client in clients.values() {
                            client.tx.send(buffer[..bytes_read].to_vec()).unwrap();
                        }
                    },
                    Err(err) => {
                      if err.kind() != std::io::ErrorKind::BrokenPipe {
                        return Err(err.into());
                      } else {
                        break;
                      }
                    }
                  }
                }
            println!("Exiting because read on pipe returned 0 bytes");
            Ok(())
        });

       
        for event in ffmpeg.iter()? {
            match event {
                // Signal threads when output is ready
                FfmpegEvent::Progress(_) if !self.ready_to_share => {
                    ready_sender.send(()).unwrap();
                    println!("message sent exiting..");
                    self.ready_to_share = true;
                    break;
                },
                _ => {
                    println!("other event: {:?}", event);
                }
            }
        }
        let handle = Mutex::new(ffmpeg);
        let list_tx_clients_clone2: Arc<Mutex<HashMap<String, Client>>> = Arc::clone(&self.list_clients);
        //LISTENER THREAD
        thread::spawn(move || {
            let mut buffer = [0; BUFFER_SIZE];
            loop{
                // Ricevi il pacchetto dal client
                let (bytes_received, client_address) = match listener_socket.recv_from(&mut buffer) {
                    Ok(res) => res,
                    Err(e) => {
                        eprintln!("Errore durante la ricezione: {}", e);
                        continue;
                    }
                };

                let message = String::from_utf8_lossy(&buffer[..bytes_received]);
                println!("Ricevuto: '{}' da {}", message, client_address);

                // Controlla se il messaggio è "START"
                if message.trim() == "START" {
                    let target_address = format!("{}:{}", client_address.ip(), client_address.port());
                    // Spawna un thread per gestire l'invio dei dati
                    let send_socket = listener_socket.clone();
                    let (tx, rx) = channel::<Vec<u8>>();

                    // Aggiungi il client alla lista
                    let mut list_guard = list_tx_clients_clone2.lock().unwrap();
                    list_guard.insert(target_address.clone(), Client{ tx, });

                    listener_socket.send_to(b"OK", &target_address).unwrap();
                    drop(list_guard);
                    println!("Client connesso: {}", target_address);
                    //Spawna un thread per inviare i dati al client
                    thread::spawn(move || {
                        loop {
                            // Ricevi i dati dal canale e inviali; quando il client viene rimosso dalla lista, il tx viene droppato e il thread termina
                            match rx.recv(){
                                Ok(data) => {send_socket.send_to(&data, &target_address).unwrap();},
                                Err(e) => {
                                    eprintln!("Errore durante la ricezione dei dati: {}", e);
                                    break;
                                }
                            }
                        }
                    });
                }

                if message.trim().starts_with("STOP"){
                    let message = message.split("\n").collect::<Vec<&str>>();
                    let ip = message[1];
                    let mut list_guard = list_tx_clients_clone2.lock().unwrap();
                    list_guard.remove(ip);
                    drop(list_guard);
                    listener_socket.send_to(b"OK", &client_address).unwrap();
                }
            }
        });

        self.handle = Some(handle);
        println!("sender will be dropped now..");
        Ok(())

    }

    pub fn stop (&mut self) {
        if let Some(ref process) = self.handle {
            let mut guard = process.lock().unwrap();

            if let Some(mut stdin) = (*guard).take_stdin() {
                writeln!(stdin, "q").unwrap();
            }
            guard.wait().expect("Failed to stop FFmpeg process");
            self.ready_to_share = false;
            println!("Screen casting fermato!");
        } else {
            println!("No casting in progress to stop.");
        }
    }
}
