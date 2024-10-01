use tokio::net::UdpSocket;
use ffmpeg_sidecar::command::FfmpegCommand;
use std::net::SocketAddr;
use std::io::Read;  
mod utils_ffmpeg;
use utils_ffmpeg::check_ffmpeg;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    
    check_ffmpeg().expect("Failed to check FFmpeg");
    let socket = UdpSocket::bind("127.0.0.1:1234").await?;  // Il server si bind sulla porta 1234
    let client_addr: SocketAddr = "127.0.0.1:1235".parse().unwrap();    // Indirizzo del client (stesso PC, porta 1235)

    let ffmpeg_command = vec![
        "-f", "avfoundation",               // Formato input per catturare lo schermo
        "-r", "30",                  // Frame rate
        "-s", "1920x1080",             // Risoluzione dello schermo
        "-capture_cursor", "1",         // Cattura il cursore
        "-i", "1:",                  // Schermo da catturare
        "-f", "rawvideo",              // Formato output
        "-preset", "ultrafast",       // Preset di compressione veloce
        "-codec:v", "libx264",      // Codec video
        "-b:v", "8M",                  // Bitrate
        "-bf", "0",                   // Nessun B-Frames
        "pipe:1",                      // Output su stdout
    ];

    // Avvia il comando FFmpeg con ffmpeg-sidecar
    let mut ffmpeg = FfmpegCommand::new().args(&ffmpeg_command).spawn().expect("Failed to start FFmpeg");

    // Attende il completamento di ffmpeg
    //ffmpeg.wait().unwrap();

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