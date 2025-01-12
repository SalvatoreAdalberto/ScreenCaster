use std::{env, path::PathBuf, process::{Command, Stdio}};
use ffmpeg_sidecar::{
    download::{check_latest_version, download_ffmpeg_package, unpack_ffmpeg},
    version::ffmpeg_version_with_path,
};

pub(crate) trait BackgroundCommand {
    fn create_no_window(&mut self) -> &mut Self;
}

impl BackgroundCommand for Command {
    /// Disable creating a new console window for the spawned process on Windows.
    /// Has no effect on other platforms. This can be useful when spawning a command
    /// from a GUI program.
    fn create_no_window(&mut self) -> &mut Self {
        #[cfg(target_os = "windows")]
        std::os::windows::process::CommandExt::creation_flags(self, 0x08000000);
        self
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
        let destination = ffmpeg_dir()?;

        println!("Downloading from: {:?}", download_url);
        let archive_path = download_ffmpeg_package(download_url, &destination)?;

        println!("Extracting...");
        unpack_ffmpeg(&archive_path, &destination)?;

        let version = ffmpeg_version()?;
        println!("Installed version: {}", version);
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
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Ok("https://evermeet.cx/ffmpeg/getrelease")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Ok("https://www.osxexperts.net/ffmpeg7arm.zip")
    } else {
        Err("Unsupported platform")
    }
}

pub fn ffmpeg_is_installed() -> bool {
    Command::new(ffmpeg_path().unwrap_or_default())
        .arg("-version")
        .create_no_window()
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or_else(|_| false)
}

pub fn ffmpeg_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let current_exe = env::current_exe().map_err(|err| {
        format!("Failed to get the path of the current executable: {}", err)
    })?;

    let temp_path = current_exe
        .parent().ok_or("Failed to navigate to parent directory.")?
        .parent().ok_or("Failed to navigate to parent directory.")?
        .parent().ok_or("Failed to navigate to parent directory.")?
        .parent().ok_or("Failed to navigate to parent directory.")?;

    Ok(temp_path.join("screen_caster/target/release/"))
}

/// The (expected) path to an FFmpeg binary adjacent to the screen_caster binary.
pub fn ffmpeg_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = ffmpeg_dir()?.join("ffmpeg");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    Ok(path)
}

pub fn ffmpeg_version() -> anyhow::Result<String> {
    ffmpeg_version_with_path(ffmpeg_path().unwrap())
  }

fn main() {
    if let Err(err) = check_ffmpeg() {
        eprintln!("Error: {}", err);
    }
}
