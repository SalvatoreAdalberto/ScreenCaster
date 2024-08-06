use std::env::args;
use std::io::{self, Write};
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::thread;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::{
    command::ffmpeg_is_installed,
    download::{check_latest_version, download_ffmpeg_package, ffmpeg_download_url, unpack_ffmpeg},
    paths::sidecar_dir,
    version::ffmpeg_version,
};

fn main() {
    if let Err(e) = check_ffmpeg() {
        eprintln!("Error checking FFmpeg: {:?}", e);
        return;
    }

    let mut command = FfmpegCommand::new();
    command.arg("-f")
        .arg("avfoundation")
        .arg("-capture_cursor")
        .arg("1")
        .arg("-i")
        .arg("1:")
        .arg("-r")
        .arg("30")
        .arg("-vf")
        .arg("crop=1000:1000:0:500")//selected_width:selected_height:horizontal_offset:vertical_offset (from top-left corner)
        .arg("-y")
        .arg("output.mp4");

    let mut output = command.spawn().unwrap();
    let mut child_stdin =  output.take_stdin().unwrap();

    let _ = thread::spawn(move || {
        println!("Press 'q' to stop the capture.");
        let mut input = String::new();
        loop {
            io::stdin().read_line(&mut input).unwrap();
            if input.trim() == "q" {
                writeln!(child_stdin, "q").unwrap();
                break;
            }
            input.clear();
        }
    }).join().unwrap();

    let _ = output.wait().unwrap();

}

fn check_ffmpeg() -> Result<(), Box<dyn std::error::Error>> {
    if ffmpeg_is_installed() {
        println!("FFmpeg is already installed! 🎉");
        println!("For demo purposes, we'll re-download and unpack it anyway.");
        println!("TIP: Use `auto_download()` to skip manual customization.");
    } else {
        // Short version without customization:
        // ffmpeg_sidecar::download::auto_download().unwrap();

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

    println!("Done! 🏁");
    Ok(())
}

fn ffmpeg_download_url_custom() -> Result<&'static str, &'static str> {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Ok("https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip")
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Ok("https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Ok("https://evermeet.cx/ffmpeg/getrelease")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Ok("https://www.osxexperts.net/ffmpeg7arm.zip")
    } else {
        Err("Unsupported platform")
    }
}
