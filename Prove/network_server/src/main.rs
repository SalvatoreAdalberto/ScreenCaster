use tokio::net::UdpSocket;
use ffmpeg_sidecar::command::FfmpegCommand;
use std::io::Read;
use std::net::SocketAddr;
mod utils_ffmpeg;
use utils_ffmpeg::check_ffmpeg;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Crea un socket UDP per inviare i dati
    let socket = UdpSocket::bind("127.0.0.1:1234").await?;  // Il server si bind sulla porta 1234
    let client_addr: SocketAddr = "127.0.0.1:1235".parse().unwrap();    // Indirizzo del client (stesso PC, porta 1235)
    check_ffmpeg().expect("Failed to check FFmpeg");
    // Crea il comando ffmpeg per catturare il video
    let ffmpeg_command = vec![
        "-f", "avfoundation",               // Formato input per catturare lo schermo
        "-r" , "30",                   // Frame rate
        "-s", "1920x1080",             // Risoluzione dello schermo
        "-capture_cursor",  "1",         // Cattura il cursore
        "-i", "1:",                  // Schermo da catturare
        "-f", "mpegts",                // Formato output
        "-codec:v", "mpeg1video",      // Codec video
        "-b:v", "1M",                  // Bitrate
        "-bf", "0",                    // Parametro per frame B
        "pipe:1",                      // Output su stdout
    ];

    // Avvia il comando FFmpeg con ffmpeg-sidecar
    let mut ffmpeg = FfmpegCommand::new().args(ffmpeg_command).spawn().unwrap();

    let mut stdout = ffmpeg.take_stdout().unwrap();

    let mut buffer = [0; 1024];
    loop {
        // Legge i dati dallo stdout di ffmpeg
        //println!("reading");
        let n = stdout.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        // Invia i dati al client tramite il socket UDP
        socket.send_to(&buffer[..n], &client_addr).await?;
    }

    Ok(())
}
