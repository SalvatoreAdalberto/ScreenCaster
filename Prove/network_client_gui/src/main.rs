mod gui;
mod video_player;
use tokio;
mod utils_ffmpeg;

#[tokio::main]
async fn main(){
    utils_ffmpeg::check_ffmpeg().expect("Failed to check FFmpeg");
    gui::run_gui();
}