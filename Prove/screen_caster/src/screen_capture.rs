use std::io::{Write};
use std::sync::{Mutex};
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::{
    command::ffmpeg_is_installed,
    download::{check_latest_version, download_ffmpeg_package, unpack_ffmpeg},
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

pub struct ScreenCapture {
    recording_process: Option<Mutex<FfmpegChild>>,
}

impl ScreenCapture {
    pub fn new() -> Self {
        ScreenCapture {
            recording_process: None,
        }
    }

    pub fn start(&mut self) {

        let crop = CropArea {
            width: 500,
            height: 1000,
            x_offset: 1000,
            y_offset: 500,
        };

        let mut rec = Self::start_recording(None);
        match rec {
            None => {
                println!("Error starting recording");
                return;
            },
            Some(_) => {}
        }
        self.recording_process = rec;
    }

    pub fn stop(&mut self) {
        if let Some(ref process) = self.recording_process {
            let mut guard = process.lock().unwrap();

            if let Some(mut stdin) = (*guard).take_stdin() {
                writeln!(stdin, "q").unwrap();
            }

            guard.wait().expect("Failed to stop FFmpeg process");

            println!("Screen casting fermato!");
        } else {
            println!("No recording in progress to stop.");
        }


        self.recording_process = None;
    }

    fn start_recording(crop: Option<CropArea>) -> Option<Mutex<FfmpegChild>> {
        let mut command = FfmpegCommand::new();

        #[cfg(target_os = "macos")]
        {
            match crop {
                Some(crop) => {
                    let com = format!("-f avfoundation -r 30 -capture_cursor 1 -i 1:  -vf crop={}:{}:{}:{} -f rawvideo -y output_5.mp4", crop.width, crop.height, crop.x_offset, crop.y_offset);
                    command.args(com.split(" "));
                }
                None => {
                    command.args("-f avfoundation -r 30 -s 1280x720 -capture_cursor 1 -i 1: -f rawvideo -y output_5.mp4".split(" "));
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            match crop {
                Some(crop) => {
                    let com = format!("-f gdigrab -framerate 30 -offset_x {} -offset_y {} -video_size {}x{} -show_region 1 -i desktop -c:v libx264 -f rawvideo -y output.mp4", crop.x_offset, crop.y_offset, crop.width, crop.height);
                    command.args(com.split(" "));
                }
                None => {
                    command.args("-f gdigrab -framerate 30 -i desktop -c:v libx264 -f rawvideo -y output.mp4".split(" "));
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            match crop {
                Some(crop) => {
                    let com = format!("-f x11grab -framerate 30 -video_size {}x{} -i :0.0+{},{} -y output.mkv", crop.width, crop.height, crop.x_offset, crop.y_offset);
                    command.args(com.split(" "));
                }
                None => {
                    command.args("-f x11grab -framerate 30 -y output.mp4".split(" "));
                }
            }
        }

        let result = command.spawn().expect("Failed to start FFmpeg");
        Some(Mutex::new(result))
    }
}

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