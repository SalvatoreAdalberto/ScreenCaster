#[cfg(not(target_os = "windows"))]
use pnet::datalink;
#[cfg(not(target_os = "windows"))]
use ipnetwork::IpNetwork;
#[cfg(target_os = "windows")]
use ipconfig::{get_adapters, OperStatus};
#[cfg(target_os = "windows")]
use if_addrs::{IfAddr, get_if_addrs};
#[cfg(target_os = "windows")]
use ipnet::Ipv4Net;
use std::net::Ipv4Addr;

use std::io;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::env;
use std::path::{Path, PathBuf};
use crate::streaming_server::CropArea;
use dirs::download_dir;
use screenshots::Screen;
use crate::error_banner::InputError;

pub const HOTKEYS_CONFIG_PATH : &str = "../config/hotkeys.txt";
pub const SAVE_DIRECTORY_CONFIG_PATH : &str = "../config/save_path.txt";

// Check if the IP address is in the same LAN as the local machine
pub fn is_ip_in_lan(ip_to_check: &str) -> Result<(), InputError> {
    let target_ip: Ipv4Addr;
    match ip_to_check.parse(){
        Ok(ip) => {
            target_ip = ip;
        },
        Err(_) => {
            return Err(InputError::NotAnIp);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let interfaces = datalink::interfaces();

        for interface in interfaces {
            for ip in interface.ips {
                // Controlla solo gli indirizzi IPv4
                if let IpNetwork::V4(network) = ip {
                    if network.contains(target_ip) {
                        return Ok(()); // L'indirizzo appartiene alla subnet
                    }
                }
            }
        }
    }

    /// On Windows, this function uses a different implementation due to the peculiarities
    /// in network configurations. Specifically, when connected to mobile hotspots, the
    /// netmask is often reported as `255.255.255.255`, which typically indicates a point-to-point connection.
    /// This function accounts for this problem.
    #[cfg(target_os = "windows")]
    {
        let interfaces = match get_if_addrs() {
            Ok(ifaces) => {
                ifaces
            }
            Err(e) => {
                return Err(InputError::NotInSameLan);
            }
        };

        for iface in interfaces {
            if let IfAddr::V4(if_v4) = iface.addr {
                let local_ip = if_v4.ip;
                let netmask = if_v4.netmask;

                let network = calculate_subnet_range(local_ip, netmask);

                if network.0 <= target_ip && target_ip <= network.1 {
                    return Ok(());
                } 
            }
        }

        let adapters = get_adapters().expect("Impossibile ottenere gli adattatori di rete");

        for adapter in adapters {
            if adapter.oper_status() == OperStatus::IfOperStatusUp {
                if let Some(ipv4) = adapter.ip_addresses().iter().find_map(|addr| match addr {
                    std::net::IpAddr::V4(ip) => Some(ip),
                    _ => None,
                }) {
                    if let Some(gateway) = adapter.gateways().into_iter().next() {
                        if let std::net::IpAddr::V4(gateway_ip) = gateway {
                
                            let subnet = Ipv4Net::new(*gateway_ip, 24).expect("Subnet non valida");

                            if subnet.contains(&target_ip) {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }
    Err(InputError::NotInSameLan)
}

// Compute the range of IP addresses in the same subnet
#[cfg(target_os = "windows")]
fn calculate_subnet_range(ip: Ipv4Addr, netmask: Ipv4Addr) -> (Ipv4Addr, Ipv4Addr) {
    let ip_u32 = u32::from(ip);
    let mask_u32 = u32::from(netmask);

    let network_start = ip_u32 & mask_u32;
    let broadcast = network_start | !mask_u32;

    (Ipv4Addr::from(network_start), Ipv4Addr::from(broadcast))
}

// Get the path of the project's source directory
pub fn get_project_src_path() -> PathBuf {
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let mut exe_dir = exe_path.parent().expect("Failed to get parent directory");

    for _ in 0..3 {
        exe_dir = exe_dir.parent().expect("Failed to get parent directory");
    }

    exe_dir.to_path_buf()
}

/// Computes the window size based on the index of the monitor to display the application on.
/// Returns the width, height, top-left x-coordinate, and top-left y-coordinate of the window.
/// The index is 1-based, where 1 corresponds to the primary monitor.
/// The index is the first command-line argument passed to the application.
/// Used only on Windows and Linux for full-screen mode.
#[cfg(not(target_os = "macos"))]
pub fn compute_window_size(index: usize)-> anyhow::Result<(u32, u32, u32, u32)> {
    let screens = Screen::all().unwrap();
    println!("{:?}", screens);

    let screen = screens[index - 1];
    let top_x = screen.display_info.x as u32;
    let top_y = screen.display_info.y as u32;
    let width = screen.display_info.width;
    let height = screen.display_info.height;

    Ok((width, height, top_x, top_y))
}

// Returns the number of screens connected to the system
pub fn count_screens() -> usize {
    let screens = Screen::all().unwrap();
    screens.len()
}

// Return the correct FFmpeg command based on the operating system, screen index, and crop area
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
                format!("-f gdigrab -framerate 30 -offset_x {} -offset_y {} -video_size {}x{} -i desktop -f mpegts pipe:1", crop.x_offset, crop.y_offset, crop.width, crop.height)
            }
            None => {
                format!("-f gdigrab -framerate 30 -offset_x {} -offset_y {} -video_size {}x{} -i desktop -f mpegts pipe:1", top_x, top_y, width, height)
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let (width, height, top_x, top_y) = compute_window_size(screen_index).unwrap();
        match crop {
            Some(crop) => {
                format!("-f x11grab -framerate 30 -video_size {}x{} -i :0.0+{},{} -f mpegts pipe:1", crop.width, crop.height, crop.x_offset, crop.y_offset)
            }
            None => {
                format!("-f x11grab -framerate 30 -video_size {}x{} -i :0.0+{},{} -f mpegts pipe:1", width, height, top_x, top_y)
            }
        }
    }

}

/// Read the hotkeys from the configuration file and return them as a tuple. 
/// If the file is empty, use the default hotkeys.
pub fn read_hotkeys()  -> io::Result<(String, String, String, String)> {
    let file = File::open(HOTKEYS_CONFIG_PATH)?;
    let start_reader = BufReader::new(&file);

    // Read the first line of the file (start)
    let start = match start_reader.lines().next() {
        Some(Ok(path)) => path,
        _ => "h".to_string(),
    };

    // Re-open the file using a new BufReader
    let file = File::open(HOTKEYS_CONFIG_PATH)?;
    let stop_reader = BufReader::new(&file);

    // Read the second line of the file (stop)
    let stop = match stop_reader.lines().nth(1) {
        Some(Ok(shortcut)) => shortcut,
        Some(Err(_err)) => {
            "j".to_string()
        }
        None => {
            "j".to_string()
        }
    };

    // Re-open the file using a new BufReader
    let file = File::open(HOTKEYS_CONFIG_PATH)?;
    let clear_reader = BufReader::new(&file);

    // Read the third line of the file (clear)
    let clear = match clear_reader.lines().nth(2) {
        Some(Ok(shortcut)) => shortcut,
        Some(Err(_err)) => {
            "k".to_string()
        }
        None => {
            "k".to_string()
        }
    };

    let file = File::open(HOTKEYS_CONFIG_PATH)?;
    let close_reader = BufReader::new(&file);

    // Read the fourth line of the file (end session)
    let close = match close_reader.lines().nth(3) {
        Some(Ok(shortcut)) => shortcut,
        Some(Err(_err)) => {
            "l".to_string()
        }
        None => {
            "l".to_string()
        }
    };

    Ok((start, stop, clear, close))
}

/// Save the hotkeys to the configuration file.
/// The hotkeys are saved in the order: start, stop, clear, close.
/// If the file does not exist, it is created.
/// If the file already exists, its content is overwritten.
pub fn save_hotkeys(key1: &str, key2: &str, key3: &str, key4: &str) -> io::Result<()> {
    let mut file = File::create(HOTKEYS_CONFIG_PATH)?;

    // Write the hotkeys to the file one for each line
    writeln!(file, "{}", key1)?;
    writeln!(file, "{}", key2)?;
    writeln!(file, "{}", key3)?;
    writeln!(file, "{}", key4)?;

    Ok(())
}

/// Get the save directory from the configuration file. If the file is empty, use the default save directory.
pub fn get_save_directory() -> io::Result<String> {
    let default_save_directory = download_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not locate the Downloads directory"))?
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid path to Downloads directory"))?
        .to_string();

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(SAVE_DIRECTORY_CONFIG_PATH)?;

    let mut reader = BufReader::new(&file);
    let mut first_line = String::new();

    if reader.read_line(&mut first_line)? == 0 {
        // File is empty, write the default save directory
        let mut writer = OpenOptions::new().write(true).open(SAVE_DIRECTORY_CONFIG_PATH)?;
        writer.write_all(default_save_directory.as_bytes())?;
        writer.write_all(b"\n")?;
        Ok(default_save_directory)
    } else {
        // Trim any whitespace or newline characters
        let savepath = first_line.trim().to_string();

        // Check if the path is valid (e.g., exists or can be used)
        if Path::new(&savepath).is_absolute() {
            Ok(savepath)
        } else {
            save_directory(&default_save_directory)?;
            Ok(default_save_directory)
        }
    }
}

/// Save the save directory to the configuration file.
/// The directory is saved as an absolute path.
/// If the file does not exist, it is created.
/// If the file already exists, its content is overwritten.
pub fn save_directory(new_directory: &str) -> io::Result<()> {
    if !Path::new(new_directory).is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The provided directory path is not absolute.",
        ));
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(SAVE_DIRECTORY_CONFIG_PATH)?;

    file.write_all(new_directory.as_bytes())?;
    file.write_all(b"\n")?;

    Ok(())
}