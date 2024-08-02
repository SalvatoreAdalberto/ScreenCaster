use scap::{
    capturer::{Area, Capturer, Options, Point, Size},
    frame::Frame,
};

use std::path::Path;

use ndarray::Array3;

use video_rs::encode::{Encoder, Settings};
use video_rs::time::Time;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    video_rs::init().unwrap();

    let output_path = "output.mp4";
    let duration: Time = Time::from_nth_of_a_second(100);
    let mut position = Time::zero();

    let framerate = 30;

    let options = Options {
        fps: framerate,
        show_cursor: true,
        show_highlight: true,
        excluded_targets: None,
        excluded_windows: None,
        output_type: scap::frame::FrameType::BGR0,
        output_resolution: scap::capturer::Resolution::_1080p,
        source_rect: Some(Area {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size { width: 1280.0, height: 720.0 },
        }),
        ..Default::default()
    };

    let mut recorder = Capturer::new(options);
    recorder.start_capture();

    let [width, height] = recorder.get_output_frame_size();

    let settings = Settings::preset_h264_yuv420p(width as usize, height as usize, false);
    let mut encoder =
        Encoder::new(Path::new(output_path), settings).expect("failed to create encoder");

    println!("🎥 Recording in progress...");
    for _ in 0..100 {
        let frame = recorder.get_next_frame()?;

        match frame {
            Frame::BGR0(frame) => {
                let frame_data = frame.data;
                let rgb_data: Vec<u8> = frame_data
                    .chunks(3)
                    .flat_map(|chunk| vec![chunk[2], chunk[1], chunk[0]])
                    .collect();

                let rgb_frame = Array3::from_shape_vec((height as usize, width as usize, 3), rgb_data)
                    .expect("Failed to create Array3 from frame data");

                encoder.encode(&rgb_frame, position).expect("failed to encode frame");
                position = position.aligned_with(duration).add();
            }
            _ => continue,
        };

    }

    encoder.finish().expect("failed to finish encoder");

    println!("?? Video saved to {}", output_path);

    Ok(())
}
