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
use ffmpeg_sidecar::child::FfmpegChild;

pub struct CropArea {
    width: u32,
    height: u32,
    x_offset: u32,
    y_offset: u32,
}

fn main() {
    if let Err(e) = check_ffmpeg() {
        eprintln!("Error checking FFmpeg: {:?}", e);
        return;
    }

    let crop = CropArea {
        width: 500,
        height: 1000,
        x_offset: 1000,
        y_offset: 500,
    };

    let mut output = start_recording(Some(crop)).unwrap();
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

fn start_recording(crop: Option<CropArea>) -> Result<FfmpegChild, &'static str> {
    let mut command = FfmpegCommand::new();

    #[cfg(target_os = "macos")]
    {
        match crop {
            Some(crop) => {
                let com = format!("-f avfoundation -capture_cursor 1 -i 1: -r 30 -vf crop={}:{}:{}:{} -y output.mp4", crop.width, crop.height, crop.x_offset, crop.y_offset);
                command.args(com.split(" "));
            }
            None => {
                command.args("-f avfoundation -capture_cursor 1 -i 1: -r 30 -y output.mp4".split(" "));
            }
        }

    }

    #[cfg(target_os = "windows")]
    {
        match crop {
            Some(crop) => {
                let com = format!("-f gdigrab -framerate 30 -offset_x {} -offset_y {} -video_size {}x{} -show_region 1 -i desktop -c:v libx264 -preset ultrafast -pix_fmt yuv420p -c:a aac -y output.mp4", crop.x_offset, crop.y_offset, crop.width, crop.height);
                command.args(com.split(" "));
            }
            None => {
                command.args("-f gdigrab -framerate 30 -i desktop -c:v libx264 -preset ultrafast -pix_fmt yuv420p -c:a aac -y output.mp4".split(" "));
            }
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        return Err("Unsupported platform");
    }

    let result = command.spawn().unwrap();
    Ok(result)
}
