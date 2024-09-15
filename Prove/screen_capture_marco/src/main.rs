mod gui;
mod screen_capture;

fn main() {
    screen_capture::check_ffmpeg().expect("Failed to check FFmpeg");
    gui::run_gui();
}