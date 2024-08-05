use screen_capure_marco::mp4::Encoder;
use scap::{
    capturer::{Area, Capturer, Options, Point, Size},
    frame::Frame,
};
use image::DynamicImage;
use image::ImageBuffer;
use ffmpeg_sidecar::{
    command::ffmpeg_is_installed,
    download::{check_latest_version, download_ffmpeg_package, ffmpeg_download_url, unpack_ffmpeg},
    paths::sidecar_dir,
    version::ffmpeg_version,
  };
fn ffmpeg_download_url_custom() -> Result<&'static str, &'static str> {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
      Ok("https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip") // working
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
      Ok("https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz") // not working
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
      Ok("https://evermeet.cx/ffmpeg/getrelease") // not working
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
      Ok("https://www.osxexperts.net/ffmpeg7arm.zip") // not working
    } else {
      Err("Unsupported platform")}
  }
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let download_url = ffmpeg_download_url_custom()?;
  let destination = sidecar_dir()?;

  // By default the download will use a `curl` command. You could also write
  // your own download function and use another package like `reqwest` instead.
  println!("Downloading from: {:?}", download_url);
  let archive_path = download_ffmpeg_package(download_url, &destination)?;
  println!("Downloaded package: {:?}", archive_path);

  // Extraction uses `tar` on all platforms (available in Windows since version 1803)
  println!("Extracting...");
  unpack_ffmpeg(&archive_path, &destination)?;
    let framerate = 30;
    let output_file = "output.mp4";

    let options = Options {
        fps: framerate,
        show_cursor: true,
        show_highlight: true,
        excluded_targets: None,
        excluded_windows: None,
        output_type: scap::frame::FrameType::BGRAFrame,
        output_resolution: scap::capturer::Resolution::_1080p,
        source_rect: Some(Area {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size { width: 1000.0, height: 1000.0 },
        }),
        ..Default::default()
    };

    let mut recorder = Capturer::new(options);
    recorder.start_capture();

    let [width,height] = recorder.get_output_frame_size();

    let mut encoder = Encoder::new(output_file, width, height, framerate)?;

    for _ in 0..100 {
        let frame = recorder.get_next_frame()?;

        match frame {
            Frame::BGRA(frame) => {
                let img = DynamicImage::ImageBgra8(ImageBuffer::from_raw(width, height, frame.data).unwrap());
                encoder.encode(&img)?;
            }
            _ => continue,
        };
    }

    encoder.close()?;

    println!("🎥 Video saved to {}", output_file);

    Ok(())
}
