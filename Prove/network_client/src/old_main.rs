use tokio::net::UdpSocket;
use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent};
use std::io::Write;
use std::process::{Command, Stdio}; 

use ffmpeg_sidecar::{
    command::ffmpeg_is_installed,
    download::{check_latest_version, download_ffmpeg_package, unpack_ffmpeg},
    paths::{sidecar_dir},
    version::ffmpeg_version,
};

pub fn check_ffmpeg() -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking FFmpeg...");
    if ffmpeg_is_installed() {
        println!("FFmpeg is already installed!");
    } else {
        match check_latest_version() {
            Ok(version) => println!("Latest available version: {}", version),
            Err(_) => println!("Skipping version check on this platform."),
        }

        let download_url = ffmpeg_download_url_custom()?;
        let destination = sidecar_dir()?;

        println!("Downloading from: {:?}", download_url);
        let archive_path = download_ffmpeg_package(download_url, &destination)?;
        println!("Downloaded package: {:?}", archive_path);

        println!("Extracting...");
        unpack_ffmpeg(&archive_path, &destination)?;

        let version = ffmpeg_version()?;
        println!("FFmpeg version: {}", version);
    }

    println!("Done!");
    Ok(())
}

fn ffmpeg_download_url_custom() -> Result<&'static str, &'static str> {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Ok("https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip")
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Ok("https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz")
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        Ok("https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-arm64-static.tar.xz")
    }
    else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Ok("https://evermeet.cx/ffmpeg/getrelease")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Ok("https://www.osxexperts.net/ffmpeg7arm.zip")
    } else {
        Err("Unsupported platform")
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    check_ffmpeg().expect("Failed to check FFmpeg");
    let mut chunks = 0;
    let mut frames = 0;
    
    FfmpegCommand::new()
        .input("udp://127.0.0.1:1235")
        .format("mpegts")
        .format("image2")
        .arg("frame-%04d.webp")
        .args(&["-vf", "scale=1280:720"])
        //.args(&["-compression_level","5"])
        //.args(&["-q:v","5"])
        //.output("pipe:1")
        .spawn()
        .unwrap()
        .iter()
        .unwrap()
        .for_each(|e|  match e {
            FfmpegEvent::OutputChunk(c) => {println!("CHUNK FOUND dim: {}B", c.len()); chunks += 1;},
            FfmpegEvent::OutputFrame(f) => {
                println!("FRAME FOUND dim: {}B", f.data.len() ); frames += 1;
            },
            _ => {println!("Other event: {:?}", e);}
        });
    // let mut i = 0;
    // loop{
    //     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    //     println!("Chunks: {}", chunks);
    //     println!("Frames: {}", frames);
    //     if i > 1000 {
    //         break;
    //     }
    //     i += 1;
    // }

    // Crea un socket UDP per ricevere i dati
    // let socket = UdpSocket::bind("127.0.0.1:1235").await?;
    // // Crea il comando FFmpeg utilizzando ffmpeg-sidecar per visualizzare il video
    // let ffmpeg_command = vec![
    //     "-f", "mpegts",               // Formato input MPEG-TS
    //     "-i", "pipe:0",               // Input da stdin
    //     "-c:v", "libx264",            // Usa il codec H.264 per il video
    //     "-preset", "fast",            // Preset di compressione veloce
    //     "-crf", "23",                 // Fattore di qualità (23 è la qualità visiva ottimale predefinita)
    //     "-f", "mp4",                  // Formato di output
    //     "-y", "output.mp4"                  // Nome del file di output
    // ];
    // // Avvia FFmpeg con ffmpeg-sidecar
    // let mut ffmpeg = FfmpegCommand::new().args(ffmpeg_command).spawn().unwrap();
    // let mut ffmpeg_stdin = ffmpeg.take_stdin().unwrap();
    // let mut buffer = [0; 1024];
    // loop {
    //     // Riceve i pacchetti UDP dal server
    //     let (n, _addr) = socket.recv_from(&mut buffer).await?;
    //     if n == 0 {
    //         break;
    //     }
    //     // Scrive i pacchetti nel stdin di FFmpeg per la visualizzazione
    //     ffmpeg_stdin.write_all(&buffer[..n]).unwrap();
    // }
    Ok(())
}
