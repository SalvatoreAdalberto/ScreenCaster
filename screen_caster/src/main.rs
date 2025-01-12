#![windows_subsystem = "windows"]

use std::sync::{Arc, Mutex};
use std::thread;
use global_hotkey::GlobalHotKeyManager;
use global_hotkey::hotkey::{HotKey, Modifiers};
use crate::hotkeys::AppState;

mod workers;
mod utils;
mod gui;
mod hotkeys;
mod gif_widget;
mod streaming_client;
mod streaming_server;
mod streamers_table;
mod error_banner;

fn main() {
    // Flag to stop the hotkey thread
    let running = Arc::new(Mutex::new(true));
    let running_clone = Arc::clone(&running);

    let app_state = Arc::new(Mutex::new(AppState::new()));
    let app_state_clone = Arc::clone(&app_state);

    // Hotkeys setup
    let (start, stop, clear, close) = utils::read_hotkeys().unwrap();

    let manager = Arc::new(Mutex::new(GlobalHotKeyManager::new().unwrap()));
    #[cfg(target_os = "macos")]
    let hotkey_record = HotKey::new(Some(Modifiers::SUPER), hotkeys::parse_key_code(&start).unwrap());
    #[cfg(not(target_os = "macos"))]
    let hotkey_record = HotKey::new(Some(Modifiers::CONTROL), hotkeys::parse_key_code(&start).unwrap());

    #[cfg(target_os = "macos")]
    let hotkey_stop = HotKey::new(Some(Modifiers::SUPER), hotkeys::parse_key_code(&stop).unwrap());
    #[cfg(not(target_os = "macos"))]
    let hotkey_stop = HotKey::new(Some(Modifiers::CONTROL), hotkeys::parse_key_code(&stop).unwrap());

    #[cfg(target_os = "macos")]
    let hotkey_clear = HotKey::new(Some(Modifiers::SUPER), hotkeys::parse_key_code(&clear).unwrap()); 
    #[cfg(not(target_os = "macos"))]
    let hotkey_clear = HotKey::new(Some(Modifiers::CONTROL), hotkeys::parse_key_code(&clear).unwrap()); 

    #[cfg(target_os = "macos")]
    let hotkey_close = HotKey::new(Some(Modifiers::SUPER), hotkeys::parse_key_code(&close).unwrap()); 
    #[cfg(not(target_os = "macos"))]
    let hotkey_close = HotKey::new(Some(Modifiers::CONTROL), hotkeys::parse_key_code(&close).unwrap());


    let id1 = Arc::new(Mutex::new(hotkey_record.id()));
    let id2 = Arc::new(Mutex::new(hotkey_stop.id()));
    let id3 = Arc::new(Mutex::new(hotkey_clear.id()));
    let id4 = Arc::new(Mutex::new(hotkey_close.id()));

    let id1_clone = id1.clone();
    let id2_clone = id2.clone();
    let id3_clone = id3.clone();
    let id4_clone = id4.clone();

    let m = manager.lock().unwrap();

    let _ = m.register(hotkey_record).unwrap();
    let _ = m.register(hotkey_stop).unwrap();
    let _ = m.register(hotkey_clear).unwrap();
    let _ = m.register(hotkey_close).unwrap();

    // Start the hotkey thread
    let handle = thread::spawn(move || {
        #[cfg(target_os = "windows")]
        hotkeys::windows_event_loop(id1_clone, id2_clone, id3_clone, id4_clone, app_state_clone, running_clone);

        #[cfg(target_os = "linux")]
        hotkeys::linux_event_loop(id1_clone, id2_clone, id3_clone, id4_clone, app_state_clone, running_clone);

        #[cfg(target_os = "macos")]
        hotkeys::macos_event_loop(id1_clone, id2_clone, id3_clone, id4_clone, app_state_clone, running_clone);
    });
    drop(m);

    // Start the GUI
    gui::run_gui(app_state, manager.clone(), id1.clone(), id2.clone(), id3.clone(), id4.clone(), hotkey_record, hotkey_stop, hotkey_clear, hotkey_close);

    // Stop the hotkey thread when the GUI is closed
    *running.lock().unwrap() = false;
    handle.join().unwrap();
}