use ffmpeg_sidecar::{
    command::ffmpeg_is_installed,
    download::{check_latest_version, download_ffmpeg_package, unpack_ffmpeg},
    paths::sidecar_dir,
    version::ffmpeg_version,
};
use ffmpeg_sidecar::command::FfmpegCommand;

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

fn main() {
    check_ffmpeg().unwrap();
}