use std::net::UdpSocket;

use std::sync::mpsc::channel;
use std::thread;
use std::sync::{Arc, Mutex};

use local_ip_address::local_ip;

use std::collections::HashMap;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::child::FfmpegChild;
use std::io::{Read, Write, BufReader};

const BUFFER_SIZE: usize = 1024;
struct Client{
    tx: std::sync::mpsc::Sender<Vec<u8>>,
}

pub struct StreamingServer {
    handle: Option<Mutex<FfmpegChild>>,
    list_clients: Arc<Mutex<HashMap<String, Client>>>,
}

impl StreamingServer {
    pub fn new() -> Self {
        StreamingServer {
            handle: None,
            list_clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start(&mut self) {
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

        let ffmpeg_command = vec![
            "-f", "avfoundation",               // Formato input per catturare lo schermo
            "-re",                  // Frame rate
            "-video_size", "1280x720",             // Risoluzione dello schermo
            "-capture_cursor", "1",         // Cattura il cursore
            "-i", "1:",                  // Schermo da catturare
            "-tune", "zerolatency",       // Tuning per bassa latenza
            "-f", "mpegts",             // Formato di output raw
            "-codec:v", "libx264",      // Codec video
            "-preset", "slow",       // Preset di compressione
            // "-b:v", "5M",                  // Bitrate
            "-crf", "28",                 // Costant Rate Factor
            "-pix_fmt", "yuv420p",       // Formato pixel
            "pipe:1"                    // Output su stdout
        ];

        // Avvia il comando FFmpeg con ffmpeg-sidecar
        let mut ffmpeg = FfmpegCommand::new().args(&ffmpeg_command).spawn().expect("Failed to start FFmpeg");
        let mut reader = BufReader::new(ffmpeg.take_stdout().unwrap());
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        let handle = Mutex::new(ffmpeg);

        //Lista dei client connessi
        let list_tx_clients_clone = Arc::clone(&self.list_clients);

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
                    let mut list_guard = list_tx_clients_clone.lock().unwrap();
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
                    let mut list_guard = list_tx_clients_clone.lock().unwrap();
                    list_guard.remove(ip);
                    drop(list_guard);
                    listener_socket.send_to(b"OK", &client_address).unwrap();
                }
            }
        });

        let list_tx_clients_clone2 = Arc::clone(&self.list_clients);
        thread::spawn(move || {
            loop {
                let n = reader.read(&mut buffer).unwrap();
                if n == 0 {
                    break;
                }
                let clients = list_tx_clients_clone2.lock().unwrap();
                for client in clients.values() {
                    client.tx.send(buffer[..n].to_vec()).unwrap();
                }
            }
        });

        self.handle = Some(handle);

    }

    pub fn stop (&mut self) {
        if let Some(ref process) = self.handle {
            let mut guard = process.lock().unwrap();

            if let Some(mut stdin) = (*guard).take_stdin() {
                writeln!(stdin, "q").unwrap();
            }

            guard.wait().expect("Failed to stop FFmpeg process");

            println!("Screen casting fermato!");
        } else {
            println!("No casting in progress to stop.");
        }
    }
}
