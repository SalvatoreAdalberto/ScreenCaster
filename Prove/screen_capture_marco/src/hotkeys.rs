use global_hotkey::{GlobalHotKeyEvent, HotKeyState, hotkey::{Code}};
use iced::Application;
use std::sync::{Arc, Mutex};

#[cfg(target_os = "windows")]
use winapi::um::winuser::{self, MSG};
#[cfg(target_os = "windows")]
use winapi::shared::winerror::WAIT_TIMEOUT;
#[cfg(target_os = "windows")]
use winapi::um::winbase::WAIT_OBJECT_0;
use crate::screen_capture;

pub struct AppState {
    pub(crate) is_sharing: bool, // Indica se siamo nella schermata di condivisione
    pub(crate) screen_capture: screen_capture::ScreenCapture, // Oggetto per la gestione della registrazione
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
pub fn macos_event_loop(id1: Arc<Mutex<u32>>, id2: Arc<Mutex<u32>>, app_state: Arc<Mutex<AppState>>, running: Arc<Mutex<bool>>) {
    loop {
        if !*running.lock().unwrap() {
            break;
        }

        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state == HotKeyState::Released {
                let mut state = app_state.lock().unwrap();
                if event.id == *id1.lock().unwrap() {
                    state.start_recording(); // Avvia la registrazione solo se siamo nella schermata di condivisione
                } else if event.id == *id2.lock().unwrap() {
                    state.stop_recording(); // Ferma la registrazione
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub fn linux_event_loop(id1: u32, id2: u32, app_state: Arc<Mutex<AppState>>, running: Arc<Mutex<bool>>) {
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
pub fn windows_event_loop(id1: u32, id2: u32, app_state: Arc<Mutex<AppState>>, running: Arc<Mutex<bool>>) {
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

pub fn parse_key_code(key: &str) -> Option<Code> {
    match key.to_uppercase().as_str() {
        "A" => Some(Code::KeyA),
        "B" => Some(Code::KeyB),
        "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD),
        "E" => Some(Code::KeyE),
        "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG),
        "H" => Some(Code::KeyH),
        "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ),
        "K" => Some(Code::KeyK),
        "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM),
        "N" => Some(Code::KeyN),
        "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP),
        "Q" => Some(Code::KeyQ),
        "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS),
        "T" => Some(Code::KeyT),
        "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV),
        "W" => Some(Code::KeyW),
        "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY),
        "Z" => Some(Code::KeyZ),
        "0" => Some(Code::Digit0),
        "1" => Some(Code::Digit1),
        "2" => Some(Code::Digit2),
        "3" => Some(Code::Digit3),
        "4" => Some(Code::Digit4),
        "5" => Some(Code::Digit5),
        "6" => Some(Code::Digit6),
        "7" => Some(Code::Digit7),
        "8" => Some(Code::Digit8),
        "9" => Some(Code::Digit9),
        _ => None,
    }
}