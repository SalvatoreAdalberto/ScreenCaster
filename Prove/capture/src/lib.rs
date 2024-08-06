pub mod mp4 {
    use std::io::Error;
    use image::{DynamicImage, GenericImageView};
    use std::io::Write;
    use std::path::Path;
    use std::process::{Command, Stdio};
    use ffmpeg_sidecar::{
        command::FfmpegCommand,
        child::FfmpegChild};

    /// MP4 encoder.
    pub struct Encoder<P: AsRef<Path>> {
        p: P,
        ffmpeg: FfmpegChild,
        width: u32,
        height: u32,
        framerate: f32,
    }

    impl<P: AsRef<Path>> Encoder<P> {
        // /// Creates a new MP4 encoder.
        // pub fn new(path: P, width: u32, height: u32, framerate: u32) -> Result<Encoder<P>, Error> {
        //     let name = path.as_ref();

        //     // ffmpegを実行するためにコマンドを組み立てる
        //     let command = |width, height, framerate, output| {
        //         format!(
        //             "ffmpeg -framerate {framerate} -f rawvideo -pix_fmt rgba -s {width}x{height} -i - -pix_fmt yuv420p -vcodec libx264 -crf 18 -preset slow -profile:v high -movflags faststart {output:?}",
        //             width = width, height = height, framerate = framerate, output = output
        //         )
        //     };

        //     // ffmpegを実行
        //     let ffmpeg = Command::new("/bin/sh")
        //         .args(&["-c", &command(width, height, framerate, name)])
        //         .stdin(Stdio::piped())
        //         .spawn()?;

        //     // 返り値のEncoder構造体は、実行中のffmpegプロセスのハンドラなどを含む
        //     Ok(Encoder {
        //         p: path,
        //         ffmpeg: ffmpeg,
        //         width: width,
        //         height: height,
        //         framerate: framerate,
        //     })
        // }

        pub fn new(path: P, width: u32, height: u32, framerate: f32) -> Result<Encoder<P>, Error> {
            let name = path.as_ref();

           

            
            let ffmpeg = FfmpegCommand::new()
                .rate(framerate)
                .size(width, height)
                .input("-")
                .args(["-f rawvideo", "-pix_fmt rgba", "-vcodec", "libx264", "-crf" ,"18" , "-preset", "slow" , "-profile:v" , "high" , "-movflags", "faststart"])
                .args(["-y", name.to_str().unwrap()])
                .spawn()?;

            // 返り値のEncoder構造体は、実行中のffmpegプロセスのハンドラなどを含む
            Ok(Encoder {
                p: path,
                ffmpeg: ffmpeg,
                width: width,
                height: height,
                framerate: framerate,
            })
        }

        // Encodes a frame.
    //     pub fn encode(&mut self, frame: &DynamicImage) -> Result<(), Error> {
    //         let (width, height) = frame.dimensions();

    //         // 入力画像のサイズがEncoderに登録されたサイズと異なる場合はエラーを返す
    //         if (width, height) != (self.width, self.height) {
    //             Err(Error::from(std::io::Error::new(
    //                 std::io::ErrorKind::Other,
    //                 "Invalid image size",
    //             )))
    //         } else {
    //             let stdin = match self.ffmpeg.stdin.as_mut() {
    //                 Some(stdin) => Ok(stdin),
    //                 None => Err(std::io::Error::new(
    //                     std::io::ErrorKind::Other,
    //                     "cannot start ffmpeg",
    //                 )),
    //             }?;

    //             // 標準入力にフレームのピクセルを流し込む
    //             stdin.write_all(&frame.to_bytes())?;
    //             Ok(())
    //         }
    //     }

    //     /// Creates a current MP4 encoder.
    //     pub fn close(&mut self) -> Result<(), Error> {
    //         self.ffmpeg.wait()?;
    //         Ok(())
    //     }
    // }
}
}