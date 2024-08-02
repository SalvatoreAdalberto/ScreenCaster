use scap::{
    capturer::{Area, Capturer, Options, Point, Size},
    frame::Frame,
};
use std::io::{Cursor, Read, Seek, SeekFrom};

use minimp4::Mp4Muxer;

use openh264::encoder::{Encoder, EncoderConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let framerate = 60;

    let options = Options {
        fps: framerate,
        show_cursor: true,
        show_highlight: true,
        excluded_targets: None,
        excluded_windows: None,
        output_type: scap::frame::FrameType::RGB,
        output_resolution: scap::capturer::Resolution::_1080p,
        source_rect: Some(Area {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size { width: 1280.0, height: 720.0 },
        }),
        ..Default::default()
    };

    let mut recorder = Capturer::new(options);
    let mut recording = Vec::new();

    recorder.start_capture();

    let [width, height] = recorder.get_output_frame_size();

    println!("🎥 Recording in progress...");
    for _ in 0..100 {
        let frame = recorder.get_next_frame()?;
        match frame {
            Frame::RGB(frame) => {
                recording.push(frame);
            }
            _ => continue,
        };
    }

    recorder.stop_capture();

    println!("🎥 Recording finished");
    let config = EncoderConfig::new(width, height);
    let mut encoder = Encoder::with_config(config).unwrap();

    let mut buf = Vec::new();

    for frame in recording {
        let mut yuv = openh264::formats::RBGYUVConverter::new(width as usize, height as usize);
        yuv.convert(&frame.data);

        let bitstream = encoder.encode(&yuv).unwrap();
        bitstream.write_vec(&mut buf);
    }

    let mut video_buffer = Cursor::new(Vec::new());
    let mut mp4muxer = Mp4Muxer::new(&mut video_buffer);
    mp4muxer.init_video(width as i32, height as i32, false, "output");
    mp4muxer.write_video(&buf);
    mp4muxer.close();

    // Some shenanigans to get the raw bytes for the video.
    video_buffer.seek(SeekFrom::Start(0)).unwrap();
    let mut video_bytes = Vec::new();
    video_buffer.read_to_end(&mut video_bytes).unwrap();

    std::fs::write("output3.mp4", &video_bytes).unwrap();


    println!("🎥 Recording finished and saved to output.mp4");

    Ok(())
}
