mod screen_capture;
mod gui;

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState, hotkey::{HotKey, Modifiers, Code}};
use iced::Application;
use std::sync::{Arc, Mutex};
use std::thread;

#[cfg(target_os = "windows")]
use winapi::um::winuser::{self, MSG};
#[cfg(target_os = "windows")]
use winapi::shared::winerror::WAIT_TIMEOUT;
#[cfg(target_os = "windows")]
use winapi::um::winbase::WAIT_OBJECT_0;

pub struct AppState {
    is_sharing: bool, // Indica se siamo nella schermata di condivisione
    screen_capture: screen_capture::ScreenCapture, // Oggetto per la gestione della registrazione
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            is_sharing: false,
            screen_capture: screen_capture::ScreenCapture::new(),
        }
    }

    pub fn start_recording(&mut self) {
        if self.is_sharing {
            self.screen_capture.start(); // Avvia la registrazione
            println!("Registrazione avviata!");
        } else {
            println!("Non siamo nella schermata di condivisione.");
        }
    }

    pub fn stop_recording(&mut self) {
        if self.is_sharing {
            self.screen_capture.stop(); // Ferma la registrazione
            println!("Registrazione fermata!");
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_event_loop(id1: u32, id2: u32, app_state: Arc<Mutex<AppState>>, running: Arc<Mutex<bool>>) {
    loop {
        if !*running.lock().unwrap() {
            break;
        }

        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state == HotKeyState::Released {
                let mut state = app_state.lock().unwrap();
                if event.id == id1 {
                    state.start_recording(); // Avvia la registrazione solo se siamo nella schermata di condivisione
                } else if event.id == id2 {
                    state.stop_recording(); // Ferma la registrazione
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn linux_event_loop(id1: u32, id2: u32, app_state: Arc<Mutex<AppState>>, running: Arc<Mutex<bool>>) {
    loop {
        if !*running.lock().unwrap() {
            break;
        }

        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            let mut state = app_state.lock().unwrap();
            if event.state == HotKeyState::Released {
                if event.id == id1 {
                    state.start_recording(); // Avvia la registrazione solo se siamo nella schermata di condivisione
                } else if event.id == id2 {
                    state.stop_recording(); // Ferma la registrazione
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn windows_event_loop(id1: u32, id2: u32, app_state: Arc<Mutex<AppState>>, running: Arc<Mutex<bool>>) {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        loop {
            if !*running.lock().unwrap() {
                break;
            }

            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                let state = app_state.lock().unwrap();
                if event.state == HotKeyState::Released {
                    if event.id == id1 {
                        state.start_recording(); // Avvia la registrazione solo se siamo nella schermata di condivisione
                    } else if event.id == id2 {
                        state.stop_recording(); // Ferma la registrazione
                    }
                }
            }

            let result = winuser::MsgWaitForMultipleObjectsEx(
                0,
                std::ptr::null(),
                0,
                winuser::QS_ALLINPUT,
                winuser::MWMO_INPUTAVAILABLE,
            );

            if result == WAIT_OBJECT_0 {
                while winuser::PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, winuser::PM_REMOVE) != 0 {
                    winuser::TranslateMessage(&msg);
                    winuser::DispatchMessageW(&msg);
                }
            } else if result == WAIT_TIMEOUT {
                continue;
            }
        }
    }
}

fn main() {
    screen_capture::check_ffmpeg().expect("Failed to check FFmpeg");
    // Flag per fermare il thread
    let running = Arc::new(Mutex::new(true));
    let running_clone = Arc::clone(&running);

    // Stato dell'applicazione
    let app_state = Arc::new(Mutex::new(AppState::new()));
    let app_state_clone = Arc::clone(&app_state);

    // Setup delle hotkey
    let manager = GlobalHotKeyManager::new().unwrap();
    #[cfg(target_os = "macos")]
    let hotkey_record = HotKey::new(Some(Modifiers::SUPER), Code::KeyH); // Cmd + H
    #[cfg(not(target_os = "macos"))]
    let hotkey_record = HotKey::new(Some(Modifiers::CONTROL), Code::KeyH); // Ctrl + H per Windows/Linux

    #[cfg(target_os = "macos")]
    let hotkey_stop = HotKey::new(Some(Modifiers::SUPER), Code::KeyF); // Cmd + F
    #[cfg(not(target_os = "macos"))]
    let hotkey_stop = HotKey::new(Some(Modifiers::CONTROL), Code::KeyF); // Ctrl + F per Windows/Linux

    let id1 = hotkey_record.id();
    let id2 = hotkey_stop.id();

    let _ = manager.register(hotkey_record).unwrap();
    let _ = manager.register(hotkey_stop).unwrap();

    // Avvio del thread per gli hotkey
    let handle = thread::spawn(move || {
        #[cfg(target_os = "windows")]
        windows_event_loop(id1, id2, app_state_clone, running_clone);

        #[cfg(target_os = "linux")]
        linux_event_loop(id1, id2, app_state_clone, running_clone);

        #[cfg(target_os = "macos")]
        macos_event_loop(id1, id2, app_state_clone, running_clone);
    });

    // Avvio dell'interfaccia grafica
    gui::run_gui(app_state);

    // Quando la GUI termina, chiudiamo anche il thread degli hotkey
    *running.lock().unwrap() = false;
    handle.join().unwrap();
}