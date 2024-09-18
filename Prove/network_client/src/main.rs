use tokio::net::UdpSocket;
use ffmpeg_sidecar::command::FfmpegCommand;
use std::io::Write;
use std::process::{Command, Stdio}; 

use ffmpeg_sidecar::{
    command::ffmpeg_is_installed,
    download::{check_latest_version, download_ffmpeg_package, unpack_ffmpeg},
    paths::{sidecar_dir,sidecar_path},
    version::ffmpeg_version,
};

// pub fn ffplay_path() -> PathBuf {
//     let default = Path::new("ffmpeg").to_path_buf();
//     match sidecar_path() {
//       Ok(sidecar_path) => match sidecar_path.exists() {
//         true => sidecar_path,
//         false => default,
//       },
//       Err(_) => default,
//     }
//   }

// pub fn ffplay_is_installed() -> bool {
//     Command::new(ffplay_path())
//       .arg("-version")
//       .create_no_window()
//       .stderr(Stdio::null())
//       .stdout(Stdio::null())
//       .status()
//       .map(|s| s.success())
//       .unwrap_or_else(|_| false)
//   }
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
    // Crea un socket UDP per ricevere i dati
    let socket = UdpSocket::bind("127.0.0.1:1235").await?;  // Il client si bind sulla porta 1235

    check_ffmpeg().expect("Failed to check FFmpeg");
    // Crea il comando FFmpeg utilizzando ffmpeg-sidecar per visualizzare il video
    let ffmpeg_command = vec![
        "-f", "mpegts",               // Formato input MPEG-TS
        "-i", "pipe:0",               // Input da stdin
        "-c:v", "libx264",            // Usa il codec H.264 per il video
        "-preset", "fast",            // Preset di compressione veloce
        "-crf", "23",                 // Fattore di qualità (23 è la qualità visiva ottimale predefinita)
        "-f", "mp4",                  // Formato di output
        "-y", "output.mp4"                  // Nome del file di output
    ];

    // Avvia FFmpeg con ffmpeg-sidecar
    let mut ffmpeg = FfmpegCommand::new().args(ffmpeg_command).spawn().unwrap();
    // let  ffmpeg = Command::new("ffplay")
    //     .arg("-f")
    //     .arg("mpegts")    // Formato input MPEG-TS
    //     .arg("-")         // Input da stdin
    //     .stdin(Stdio::piped())
    //     .spawn()
    //     .expect("Failed to start ffplay");
    let mut ffmpeg_stdin = ffmpeg.take_stdin().unwrap();

    let mut buffer = [0; 1024];
    loop {
        // Riceve i pacchetti UDP dal server
        let (n, _addr) = socket.recv_from(&mut buffer).await?;
        if n == 0 {
            break;
        }
        // Scrive i pacchetti nel stdin di FFmpeg per la visualizzazione
        ffmpeg_stdin.write_all(&buffer[..n]).unwrap();
    }
    Ok(())
}
