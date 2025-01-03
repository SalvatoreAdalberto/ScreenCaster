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

const BUFFER_SIZE: usize = 1024;
struct Client{
    tx: std::sync::mpsc::Sender<Vec<u8>>,
}

pub struct StreamingServer {
    handle: Option<Mutex<FfmpegChild>>,
    list_clients: Arc<Mutex<HashMap<String, Client>>>,
    control: Arc<(Mutex<bool>, Condvar)>, // Aggiunta per controllare la terminazione
    threads: Vec<thread::JoinHandle<()>>,
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
            handle: None,
            list_clients: Arc::new(Mutex::new(HashMap::new())),
            control: Arc::new((Mutex::new(false), Condvar::new())), // Inizializzazione
            threads: Vec::new(),
        }
    }

    
    pub fn start(&mut self, screen_index: usize, share_mode: ShareMode) {

        {
            let (lock, cvar) = &*self.control;
            let mut terminate = lock.lock().unwrap();
            *terminate = false;
            cvar.notify_all(); // Notifica tutti i thread
        }

        let command ;

        if share_mode == ShareMode::CropArea {
            let exe_path = utils::get_project_src_path();
            let file_path;
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

        let ffmpeg_command = command.split(" ").collect::<Vec<&str>>();

        // Avvia il comando FFmpeg con ffmpeg-sidecar
        let mut ffmpeg = FfmpegCommand::new().args(&ffmpeg_command).spawn().expect("Failed to start FFmpeg");
        let mut reader = BufReader::new(ffmpeg.take_stdout().unwrap());
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        let handle = Mutex::new(ffmpeg);

        let control = Arc::clone(&self.control);
        let control_clone = Arc::clone(&self.control);

        //Lista dei client connessi
        let list_tx_clients_clone = Arc::clone(&self.list_clients);

        listener_socket.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

        //LISTENER THREAD
        let h = thread::spawn(move || {
            let mut buffer = [0; BUFFER_SIZE];
            let (lock, cvar) = &*control;

            loop{

                if *lock.lock().unwrap() {
                    println!("Terminazione del listener socket.");
                    break;
                }

                // Ricevi il pacchetto dal client
                let (bytes_received, client_address) = match listener_socket.recv_from(&mut buffer) {
                    Ok(res) => res,
                    Err(_) => {
                        continue
                    }
                };

                let message = String::from_utf8_lossy(&buffer[..bytes_received]);
                println!("Ricevuto: '{}' da {}", message, client_address);
                let target_address = format!("{}:{}", client_address.ip(), client_address.port());


                let mut list_guard = list_tx_clients_clone.lock().unwrap();
                // Controlla se il messaggio è "START"
                if message.trim() == "START"{
                    if !list_guard.contains_key(&target_address.clone()){

                        let send_socket = listener_socket.clone();
                        let (tx, rx) = channel::<Vec<u8>>();

                        list_guard.insert(target_address.clone(), Client{ tx, });
                        listener_socket.send_to(b"OK", &target_address).unwrap();
                        
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
                    }else{
                        listener_socket.send_to(b"OK", &target_address).unwrap();
                    }
                }
                if message.trim().starts_with("STOP"){
                    let message = message.split("\n").collect::<Vec<&str>>();
                    let ip = message[1];
                    list_guard.remove(ip);
                    listener_socket.send_to(b"OK", &client_address).unwrap();
                }
                drop(list_guard);
            }
            println!("Listener terminato.");
            cvar.notify_all(); // Notifica che il listener è terminato
        });

        self.threads.push(h);

        let list_tx_clients_clone2 = Arc::clone(&self.list_clients);
        let h = thread::spawn(move || {
            let (lock, cvar) = &*control_clone;

            loop {
                if *lock.lock().unwrap() {
                    println!("Terminazione del sender thread.");
                    break;
                }

                let n = reader.read(&mut buffer).unwrap();
                if n == 0 {
                    break;
                }
                let clients = list_tx_clients_clone2.lock().unwrap();
                for client in clients.values() {
                    client.tx.send(buffer[..n].to_vec()).unwrap();
                }
            }

            println!("Sender thread terminato.");
            cvar.notify_all(); // Notifica che il sender è terminato
        });

        self.threads.push(h);

        self.handle = Some(handle);

    }

    pub fn stop (&mut self) {
        if let Some(ref process) = self.handle {
            let mut guard = process.lock().unwrap();

            if let Some(mut stdin) = (*guard).take_stdin() {
                writeln!(stdin, "q").unwrap();
            }

            guard.wait().expect("Failed to stop FFmpeg process");

            {
                let (lock, cvar) = &*self.control;
                let mut terminate = lock.lock().unwrap();
                *terminate = true;
                cvar.notify_all(); // Notifica tutti i thread
            }

            for h in self.threads.drain(..) {
                h.join().unwrap();
            }

            println!("Screen casting fermato!");
        } else {
            println!("No casting in progress to stop.");
        }
    }
}