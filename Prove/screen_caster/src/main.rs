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

fn main() {
    utils::check_ffmpeg().expect("Failed to check FFmpeg");
    // Flag per fermare il thread
    let running = Arc::new(Mutex::new(true));
    let running_clone = Arc::clone(&running);

    // Stato dell'applicazione
    let app_state = Arc::new(Mutex::new(AppState::new()));
    let app_state_clone = Arc::clone(&app_state);

    let (start, stop, clear, close) = utils::read_hotkeys().unwrap();

    // Setup delle hotkey
    let manager = Arc::new(Mutex::new(GlobalHotKeyManager::new().unwrap()));
    #[cfg(target_os = "macos")]
    let hotkey_record = HotKey::new(Some(Modifiers::SUPER), hotkeys::parse_key_code(&start).unwrap()); // Cmd + H
    #[cfg(not(target_os = "macos"))]
    let hotkey_record = HotKey::new(Some(Modifiers::CONTROL), hotkeys::parse_key_code(&start).unwrap()); // Ctrl + H per Windows/Linux

    #[cfg(target_os = "macos")]
    let hotkey_stop = HotKey::new(Some(Modifiers::SUPER), hotkeys::parse_key_code(&stop).unwrap()); // Cmd + J
    #[cfg(not(target_os = "macos"))]
    let hotkey_stop = HotKey::new(Some(Modifiers::CONTROL), hotkeys::parse_key_code(&stop).unwrap()); // Ctrl + J per Windows/Linux

    #[cfg(target_os = "macos")]
    let hotkey_clear = HotKey::new(Some(Modifiers::SUPER), hotkeys::parse_key_code(&clear).unwrap()); // Cmd + K
    #[cfg(not(target_os = "macos"))]
    let hotkey_clear = HotKey::new(Some(Modifiers::CONTROL), hotkeys::parse_key_code(&clear).unwrap()); // Ctrl + K per Windows/Linux

    #[cfg(target_os = "macos")]
    let hotkey_close = HotKey::new(Some(Modifiers::SUPER), hotkeys::parse_key_code(&close).unwrap()); // Cmd + L
    #[cfg(not(target_os = "macos"))]
    let hotkey_close = HotKey::new(Some(Modifiers::CONTROL), hotkeys::parse_key_code(&close).unwrap()); // Ctrl + L per Windows/Linux


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

    // Avvio del thread per gli hotkey
    let handle = thread::spawn(move || {
        #[cfg(target_os = "windows")]
        hotkeys::windows_event_loop(id1_clone, id2_clone, id3_clone, id4_clone, app_state_clone, running_clone);

        #[cfg(target_os = "linux")]
        hotkeys::linux_event_loop(id1_clone, id2_clone, id3_clone, id4_clone, app_state_clone, running_clone);

        #[cfg(target_os = "macos")]
        hotkeys::macos_event_loop(id1_clone, id2_clone, id3_clone, id4_clone, app_state_clone, running_clone);
    });
    drop(m);

    // Avvio dell'interfaccia grafica
    gui::run_gui(app_state, manager.clone(), id1.clone(), id2.clone(), id3.clone(), id4.clone(), hotkey_record, hotkey_stop, hotkey_clear, hotkey_close);

    // Quando la GUI termina, chiudiamo anche il thread degli hotkey
    *running.lock().unwrap() = false;
    handle.join().unwrap();
}