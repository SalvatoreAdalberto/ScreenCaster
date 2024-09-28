use tokio::net::UdpSocket;
use ffmpeg_sidecar::command::FfmpegCommand;
use std::net::SocketAddr;
mod utils_ffmpeg;
use utils_ffmpeg::check_ffmpeg;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    check_ffmpeg().expect("Failed to check FFmpeg");

    let ffmpeg_command = vec![
        "-f", "avfoundation",               // Formato input per catturare lo schermo
        "-r", "30",                  // Frame rate
        "-s", "1280x720",             // Risoluzione dello schermo
        "-capture_cursor", "1",         // Cattura il cursore
        "-i", "1:",                  // Schermo da catturare
        "-f", "rawvideo",              // Formato output
        "-codec:v", "libx264",      // Codec video
        "-preset", "ultrafast",       // Preset di compressione veloce
        "-b:v", "3M",                  // Bitrate
        "udp://127.0.0.1:1235",                      // Output su stdout
    ];

    // Avvia il comando FFmpeg con ffmpeg-sidecar
    let mut ffmpeg = FfmpegCommand::new().args(&ffmpeg_command).spawn().expect("Failed to start FFmpeg");

    // Attende il completamento di ffmpeg
    ffmpeg.wait().unwrap();

    Ok(())
}