use tokio::net::UdpSocket;

use tokio::{task, sync::broadcast};
use std::sync::Arc;

use ffmpeg_sidecar::command::FfmpegCommand;
use std::net::SocketAddr;
use std::io::Read;  
mod utils_ffmpeg;
use utils_ffmpeg::check_ffmpeg;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    check_ffmpeg().expect("Failed to check FFmpeg");

    let ffmpeg_command = vec![
        "-f", "avfoundation",               // Formato input per catturare lo schermo
        "-r", "30",                  // Frame rate
        "-s", "1920x1080",             // Risoluzione dello schermo
        "-capture_cursor", "1",         // Cattura il cursore
        "-i", "1:",                  // Schermo da catturare
        "-f", "rawvideo",              // Formato output
        "-preset", "ultrafast",       // Preset di compressione veloce
        "-codec:v", "libx264",      // Codec video
        "-b:v", "2M",                  // Bitrate
        "-bf", "0",                   // Nessun B-Frames
        "pipe:1",                      // Output su stdout
    ];

    // Avvia il comando FFmpeg con ffmpeg-sidecar
    let mut ffmpeg = FfmpegCommand::new().args(&ffmpeg_command).spawn().expect("Failed to start FFmpeg");

    //Server Socket Binding
    let socket = UdpSocket::bind("192.168.1.13:1234").await?;  // Il server si bind sulla porta 1234

    //Client address (static clients -> clients should be executed first, the number of clients is fixed)
    let client_addr: SocketAddr = "192.168.1.13:1235".parse().unwrap();    
    let client_addr1: SocketAddr = "192.168.1.13:1236".parse().unwrap();   
    let clients = vec![client_addr1, client_addr];

// SERVER UDP WITH TOKIO TASKS FOR MULTIPLE CLIENTS, CHANNEL FOR BUFFERING
/*******************************/
    let (tx, _) = broadcast::channel::<Vec<u8>>(1000); // Canale per bufferizzare i dati
    let mut buffer = [0; 2048];
    let stdout = Arc::new(tokio::sync::Mutex::new(ffmpeg.take_stdout().unwrap()));
    let socket_arc = Arc::new(tokio::sync::Mutex::new(socket));

    // Creare un task per ogni client per gestire la trasmissione dei dati
    let mut tasks = vec![];
    for client in clients {
        let mut rx = tx.subscribe();  // Ogni task riceve un nuovo subscriber dal canale broadcast
        let local_socket = Arc::clone(&socket_arc);
        
        let handle = task::spawn(async move {
            while let Ok(data) = rx.recv().await {
                let lock_socket = local_socket.lock().await;
                if let Err(e) = lock_socket.send_to(&data, client).await {
                    eprintln!("Failed to send data to client: {:?}", e);
                    break;
                }
            }
        });

        tasks.push(handle);
    }

    // Task per bufferizzare i dati in arrivo da FFmpeg
    let handle = task::spawn(async move {
        loop {
            let mut lock = stdout.lock().await;
            let n = lock.read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            if tx.send(buffer[..n].to_vec()).is_err() {
                eprintln!("No clients available to receive data");
                break;
            }
        }
    });

    tasks.push(handle);

    // Attendere che tutti i task completino
    for task in tasks {
        let _ = task.await;
    }

// SERVER UDP WITH TOKIO TASKS FOR MULTIPLE CLIENTS (NOT WORKING)
/*******************************/
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

//SERER UDP SINGLE THREAD
/*******************************/
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