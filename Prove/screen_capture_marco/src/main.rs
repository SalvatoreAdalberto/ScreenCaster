use screen_capure_marco::mp4::Encoder;
use scap::{
    capturer::{Area, Capturer, Options, Point, Size},
    frame::Frame,
};
use image::DynamicImage;
use image::ImageBuffer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
