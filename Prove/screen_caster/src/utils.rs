use ipnetwork::IpNetwork;
use pnet::datalink;
use std::io::BufRead;
use std::env;
use std::path::PathBuf;
use druid::Screen;
use ffmpeg_sidecar::{
    command::ffmpeg_is_installed,
    download::{check_latest_version, download_ffmpeg_package, unpack_ffmpeg},
    paths::sidecar_dir,
    version::ffmpeg_version,
};
use ffmpeg_sidecar::command::FfmpegCommand;
use iced::advanced::graphics::image::image_rs::write_buffer_with_format;
use crate::streaming_server::CropArea;

pub const STREAMERS_LIST_PATH : &str = "../config/streamers_list.txt";

pub fn is_ip_in_lan(ip_to_check: &str) -> bool {
    // Converti l'indirizzo target in un oggetto IPv4
    let target_ip: std::net::Ipv4Addr = ip_to_check
        .parse()
        .expect("Indirizzo IP non valido");

    // Ottieni le interfacce di rete
    let interfaces = datalink::interfaces();

    for interface in interfaces {
        for ip in interface.ips {
            // Controlla solo gli indirizzi IPv4
            if let IpNetwork::V4(network) = ip {
                if network.contains(target_ip) {
                    return true; // L'indirizzo appartiene alla subnet
                }
            }
        }
    }

    false // Nessuna corrispondenza trovata
}

pub fn get_streamers_map() -> std::collections::HashMap<String, String> {
    let mut streamers_map = std::collections::HashMap::new();
    let file = std::fs::File::open(STREAMERS_LIST_PATH).expect("File non trovato");
    let mut reader = std::io::BufReader::new(file);
    let mut line = String::new();

    while let Ok(_) = reader.read_line(&mut line) {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            break;
        }
        println!("{:?}", parts);
        let key = parts[0].to_string();
        let value = parts[1].to_string();
        streamers_map.insert(key, value);
        line.clear();
    }

    println!("{:?}", streamers_map);
    streamers_map

}

pub fn get_project_src_path() -> PathBuf {
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let mut exe_dir = exe_path.parent().expect("Failed to get parent directory");

    for _ in 0..3 {
        exe_dir = exe_dir.parent().expect("Failed to get parent directory");
    }

    exe_dir.to_path_buf()
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

pub fn compute_window_size(index: usize) -> anyhow::Result<(f64, f64, f64, f64)> {
    let screens = Screen::get_monitors();
    println!("{:?}", screens);
    let width = screens.to_vec()[index].virtual_rect().width();
    let height = screens.to_vec()[index].virtual_rect().height();
    let top_x = screens.to_vec()[index].virtual_rect().x0;
    let top_y = screens.to_vec()[index].virtual_work_rect().y0;
    Ok((width, height-0.5, top_x, top_y+0.5))
}

pub fn count_screens() -> usize {
    let screens = Screen::get_monitors();
    screens.len()
}

pub fn get_ffmpeg_command(screen_index:usize, crop: Option<CropArea>) -> String {

    #[cfg(target_os = "macos")]
    {
        match crop {
            Some(crop) => {
                format!("-f avfoundation -re -video_size 1280x720 -capture_cursor 1 -i {}: -vf crop={}:{}:{}:{} -tune zerolatency -f mpegts -codec:v libx264 -preset slow -crf 28 -pix_fmt yuv420p pipe:1", screen_index, crop.width, crop.height, crop.x_offset, crop.y_offset)

            }
            None => {
                format!("-f avfoundation -re -video_size 1280x720 -capture_cursor 1 -i {}: -tune zerolatency -f mpegts -codec:v libx264 -preset slow -crf 28 -pix_fmt yuv420p pipe:1", screen_index)
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let (width, height, top_x, top_y) = compute_window_size(screen_index).unwrap();
        match crop {
            Some(crop) => {
                format!("-f gdigrab -framerate 30 -offset_x {} -offset_y {} -video_size {}x{} -i desktop -capture_cursor 1 -tune zerolatency -f mpegts -codec:v libx264 -preset slow -crf 28 -pix_fmt yuv420p pipe:1", crop.x_offset, crop.y_offset, crop.width, crop.height)
            }
            None => {
                format!("-f gdigrab -framerate 30 -offset_x {} -offset_y {} -video_size {}x{} -i desktop -capture_cursor 1 -tune zerolatency -f mpegts -codec:v libx264 -preset slow -crf 28 -pix_fmt yuv420p pipe:1", top_x, top_y-0.5, width, height+0.5)
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let (width, height, top_x, top_y) = compute_window_size(screen_index).unwrap();
        match crop {
            Some(crop) => {
                format!("ffmpeg -f x11grab -framerate 30 -video_size {}x{} -i :0.0+{},{} -draw_mouse 1 -tune zerolatency -f mpegts -codec:v libx264 -preset faster -crf 28 -pix_fmt yuv420p pipe:1", crop.width, crop.height, crop.x_offset, crop.y_offset)
            }
            None => {
                format!("ffmpeg -f x11grab -framerate 30 -video_size {}x{} -i :0.0+{},{} -draw_mouse 1 -tune zerolatency -f mpegts -codec:v libx264 -preset faster -crf 28 -pix_fmt yuv420p pipe:1", width, height+0.5, top_x, top_y-0.5)
            }
        }
    }

}