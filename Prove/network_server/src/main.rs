use std::net::UdpSocket;

use std::sync::mpsc::channel;
use std::thread;
use std::sync::{Arc, Mutex};

//use tokio::net::UdpSocket;
// use tokio::{task, sync::broadcast};
// use std::sync::Arc;
use std::process::Stdio;
use ffmpeg_sidecar::command::FfmpegCommand;
use std::net::SocketAddr;
use std::io::{Read, Write, BufReader};  
mod utils_ffmpeg;
use utils_ffmpeg::check_ffmpeg;

//#[tokio::main]
fn main() -> std::io::Result<()> {
    check_ffmpeg().expect("Failed to check FFmpeg");

    let ffmpeg_command = vec![
        "-f", "avfoundation",               // Formato input per catturare lo schermo
        "-r", "30",                  // Frame rate
        "-video_size", "1280x720",             // Risoluzione dello schermo
        "-capture_cursor", "1",         // Cattura il cursore
        "-i", "1:",                  // Schermo da catturare
        "-preset", "ultrafast",       // Preset di compressione 
        "-tune", "zerolatency",       // Tuning per bassa latenza
        "-f", "mpegts",             // Formato di output raw
        "-omit_video_pes_length", "0", // Lunghezza PES
        "-codec:v", "libx264",      // Codec video
        "-b:v", "1M",                  // Bitrate
        "-crf", "23",                 // Costant Rate Factor
        "-pix_fmt", "yuv420p",       // Formato pixel
        "-bf", "0",                   // Nessun B-Frames
        "-vf", "unsharp=3:3:1.0",          // Filtri per migliorare la qualità
        "pipe:1"                    // Output su stdout
    ];

    // Avvia il comando FFmpeg con ffmpeg-sidecar
    let mut ffmpeg = FfmpegCommand::new().args(&ffmpeg_command).as_inner_mut().stderr(Stdio::piped()).spawn().expect("Failed to start FFmpeg");

    //Server Socket Binding
    let socket = UdpSocket::bind("192.168.0.100:1935").expect("Failed to bind socket");  // Il server si bind sulla porta 1234

    //Client address (static clients -> clients should be executed first, the number of clients is fixed)
    let client_addr: SocketAddr = "192.168.0.100:1936".parse().unwrap();    
    let client_addr1: SocketAddr = "192.168.1.13:1236".parse().unwrap();   
    let clients = vec![client_addr];

    let socket_arc = Arc::new(socket);

    let mut reader = BufReader::new(ffmpeg.stdout.take().unwrap());
    let mut buffer: [u8;1024] = [0; 1024];
    let mut threads = vec![];
    let mut txs = vec![];
    // Creare un thread per ogni client per gestire la trasmissione dei dati
    for client in clients{
        let (tx, rx) = channel::<Vec<u8>>();
        let local_socket = Arc::clone(&socket_arc);
        txs.push(tx);
        let handle = thread::spawn(move || {
            loop {
                let data = rx.recv().unwrap();
                local_socket.send_to(&data, client).unwrap();
            }
        });
        threads.push(handle);
    }  
    let mut stderr_record = ffmpeg.stderr.take().unwrap();
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
    loop {
            let n = reader.read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            for tx in &txs {
                tx.send(buffer[..n].to_vec()).unwrap();
            }
        }
    for thread in threads {
        thread.join().unwrap();
    }
    
/**************************************************/
// SERVER WITH FFMPEG COMMANDS _ NOT WORKING
    // let client1 = "udp://192.168.1.147:1935";
    // let client2 = "udp://192.168.1.95:1936";
    // let mut ffmpeg_send1 = FfmpegCommand::new()
    //     .input("-")
    //     .args(&["-f", "rawvideo"])
    //     .output(client1)
    //     .as_inner_mut().stdin(Stdio::piped()).stderr(Stdio::piped())
    //     .spawn().expect("Failed to start FFmpeg_client_1");
    // let mut client1 = ffmpeg_send1.stdin.take().unwrap();
    // // let mut client2 = ffmpeg_send2.take_stdin().unwrap();
    // let mut clients = vec![client1];
    // let mut stdout = ffmpeg.stdout.take().unwrap();
    // let mut stderr_record = ffmpeg.stderr.take().unwrap();
    // let mut stderr_send = ffmpeg_send1.stderr.take().unwrap();
    // thread::spawn(move || {
    //     let mut buffer = [0; 256];
    //     loop {
    //         let n = stderr_record.read(&mut buffer).unwrap();
    //         if n == 0 {
    //             break;
    //         }
    //         eprintln!("Record Process: {}", String::from_utf8_lossy(&buffer[..n]));
    //     }
    // });
    // thread::spawn(move || {
    //     let mut buffer = [0; 256];
    //     loop {
    //         let n = stderr_send.read(&mut buffer).unwrap();
    //         if n == 0 {
    //             break;
    //         }
    //         eprintln!("Transmission Process: {}", String::from_utf8_lossy(&buffer[..n]));
    //     }
    // });
    // let mut buffer = [0; 1024];
    // println!("Server started"); 
    // loop {
    //     let n = stdout.read(&mut buffer).unwrap();
    //     if n == 0 {
    //         break;
    //     }
    //     for client in clients.iter_mut() {
    //         client.write_all(&buffer[..n]).unwrap();
    //         client.flush().unwrap();
    //     }
    // }
/****************************************/
// SERVER UDP WITH OS THREADS CONCURRENCY
    // let socket_arc = Arc::new(Mutex::new(socket));
    // let stdout = Arc::new(Mutex::new(ffmpeg.take_stdout().unwrap()));
    // let mut buffer = [0; 1024];
    // let mut threads = vec![];
    // let mut txs = vec![];
    // // Creare un thread per ogni client per gestire la trasmissione dei dati
    // for client in clients{
    //     let (tx, rx) = channel::<Vec<u8>>();
    //     let local_socket = Arc::clone(&socket_arc);
    //     txs.push(tx);
    //     let handle = thread::spawn(move || {
    //         loop {
    //             let data = rx.recv().unwrap();
    //             local_socket.lock().unwrap().send_to(&data, client).unwrap();
    //         }
    //     });
    //     threads.push(handle);
    // }  
    // // Task per bufferizzare i dati in arrivo da FFmpeg
    // let handle = thread::spawn(move || {
    //     loop {
    //         let mut lock = stdout.lock().unwrap();
    //         let n = lock.read(&mut buffer).unwrap();
    //         if n == 0 {
    //             break;
    //         }
    //         for tx in &txs {
    //             tx.send(buffer[..n].to_vec()).unwrap();
    //         }
    //     }
    // });
    // threads.push(handle);
    // for thread in threads {
    //     thread.join().unwrap();
    // }
/********************************/
// SERVER UDP WITH TOKIO TASKS FOR MULTIPLE CLIENTS, CHANNEL FOR BUFFERING
    // let (tx, _) = broadcast::channel::<Vec<u8>>(1000); // Canale per bufferizzare i dati
    // let mut buffer = [0; 512];
    // let stdout = Arc::new(tokio::sync::Mutex::new(ffmpeg.take_stdout().unwrap()));
    // let socket_arc = Arc::new(tokio::sync::Mutex::new(socket));
    // // Creare un task per ogni client per gestire la trasmissione dei dati
    // let mut tasks = vec![];
    // for client in clients {
    //     let mut rx = tx.subscribe();  // Ogni task riceve un nuovo subscriber dal canale broadcast
    //     let local_socket = Arc::clone(&socket_arc);   
    //     let handle = task::spawn(async move {
    //         while let Ok(data) = rx.recv().await {
    //             let lock_socket = local_socket.lock().await;
    //             if let Err(e) = lock_socket.send_to(&data, client).await {
    //                 eprintln!("Failed to send data to client: {:?}", e);
    //                 break;
    //             }
    //         }
    //     });
    //     tasks.push(handle);
    // }
    // // Task per bufferizzare i dati in arrivo da FFmpeg
    // let handle = task::spawn(async move {
    //     loop {
    //         let mut lock = stdout.lock().await;
    //         let n = lock.read(&mut buffer).unwrap();
    //         if n == 0 {
    //             break;
    //         }
    //         if tx.send(buffer[..n].to_vec()).is_err() {
    //             eprintln!("No clients available to receive data");
    //             break;
    //         }
    //     }
    // });
    // tasks.push(handle);
    // // Attendere che tutti i task completino
    // for task in tasks {
    //     let _ = task.await;
    // }
/*******************************/
// SERVER UDP WITH TOKIO TASKS FOR MULTIPLE CLIENTS (NOT WORKING)
    // // Take the stdout of the ffmpeg process
    // let stdout = Arc::new(Mutex::new(ffmpeg.take_stdout().unwrap()));
    // let mut buffer = [0; 1024];
    // // Create a task for each client to handle the data transmission
    // let mut tasks = vec![];
    // let socket_arc = Arc::new(tokio::sync::Mutex::new(socket));
    // for client in clients {
    //     // Spawn a new task for each client connection
    //     let local_stdout = stdout.clone(); // Clone stdout for each task
    //     let local_socket = socket_arc.clone(); // Clone the socket for each task
    //     let handle = task::spawn(async move {
    //         loop {
    //             let mut lock = local_stdout.lock().await;
    //             let n = lock.(&mut buffer).unwrap();
    //             drop(lock);
    //             if n == 0 {
    //                 break;
    //             }
    //             // Send data to the client via the TCP socket
    //             let lock_socket = local_socket.lock().await;
    //             if let Err(e) = lock_socket.send_to(&buffer[..n], client).await {
    //                 eprintln!("Failed to send data to client: {:?}", e);
    //                 break;
    //             }
    //         }
    //     });
    //     tasks.push(handle);
    // }
    // // Wait for all tasks to complete
    // for task in tasks {
    //     let _ = task.await;
    // }
/*******************************/
//SERER UDP SINGLE THREAD
    // let mut stdout = ffmpeg.take_stdout().unwrap();
    // let mut buffer = [0; 1024];
    // loop {
    //     // Legge i dati dallo stdout di ffmpeg
    //     //println!("reading");
    //     let n = stdout.read(&mut buffer).unwrap();
    //     if n == 0 {
    //         break;
    //     }
    //     // Invia i dati al client tramite il socket UDP
    //      for client in &clients {
    //         socket.send_to(&buffer[..n], client).await?;
    //      }
    //     //socket.send_to(&buffer[..n], &client_addr).await?;
    // }
/*******************************/

    Ok(())
}