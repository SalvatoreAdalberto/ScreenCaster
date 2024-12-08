use std::io::Write;
use global_hotkey::{GlobalHotKeyEvent, HotKeyState, hotkey::{Code}};
use std::sync::{Arc, Mutex};

#[cfg(target_os = "windows")]
use winapi::um::winuser::{self, MSG};
#[cfg(target_os = "windows")]
use winapi::shared::winerror::WAIT_TIMEOUT;
#[cfg(target_os = "windows")]
use winapi::um::winbase::WAIT_OBJECT_0;
use crate::streaming_server;
use crate::gui::ShareMode;



pub struct AppState {
    pub(crate) is_sharing: bool, // Indica se siamo nella schermata di condivisione
    pub(crate) streaming_server: streaming_server::StreamingServer, // Oggetto per la gestione della registrazione
    pub(crate) share_mode: ShareMode, // Modalità di condivisione
    pub(crate) screen_index: usize, // Indice dello schermo da condividere
    pub(crate) annotation_stdin: Option<std::process::ChildStdin>, // Stdin per l'invio delle annotazioni
    pub(crate) is_drawing: bool, // Indica se siamo in modalità di disegno
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            is_sharing: false,
            streaming_server: streaming_server::StreamingServer::new(),
            share_mode: ShareMode::Fullscreen,
            screen_index: 1,
            annotation_stdin: None,
            is_drawing: false,
        }
    }

    pub fn start(&mut self) {
        if self.is_sharing {
            self.streaming_server.start(self.screen_index, self.share_mode); // Avvia la registrazione
            println!("Registrazione avviata!");
        } else {
            println!("Non siamo nella schermata di condivisione.");
        }
    }

    pub fn stop(&mut self) {
        if self.is_sharing {
            self.streaming_server.stop(); // Ferma la registrazione
            if let Some(ref mut std) = self.annotation_stdin {
                if writeln!(std, "quit").is_ok() {
                    println!("Annotation closed");
                    self.annotation_stdin = None;
                } else {
                    eprintln!("Lo stdin è chiuso.");
                    self.annotation_stdin = None;
                }
            } else {
                eprintln!("Lo stdin non è disponibile.");
            }
        }
    }

    pub fn clear(&mut self) {
        if let Some(ref mut std) = self.annotation_stdin {
            if writeln!(std, "clear").is_ok() {
                println!("Annotation cleared");
            } else {
                eprintln!("Lo stdin è chiuso.");
                self.annotation_stdin = None;
            }
        } else {
            eprintln!("Lo stdin non è disponibile.");
        }
    }
}

#[cfg(target_os = "macos")]
pub fn macos_event_loop(id1: Arc<Mutex<u32>>, id2: Arc<Mutex<u32>>, id3: Arc<Mutex<u32>>, app_state: Arc<Mutex<AppState>>, running: Arc<Mutex<bool>>) {
    loop {
        if !*running.lock().unwrap() {
            break;
        }

        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state == HotKeyState::Released {
                let mut state = app_state.lock().unwrap();
                if event.id == *id1.lock().unwrap() {
                    state.start(); // Avvia la registrazione solo se siamo nella schermata di condivisione
                } else if event.id == *id2.lock().unwrap() {
                    state.stop(); // Ferma la registrazione
                } else if event.id == *id3.lock().unwrap() {
                    state.clear();
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub fn linux_event_loop(id1: Arc<Mutex<u32>>, id2: Arc<Mutex<u32>>, id3: Arc<Mutex<u32>>, app_state: Arc<Mutex<AppState>>, running: Arc<Mutex<bool>>) {
    loop {
        if !*running.lock().unwrap() {
            break;
        }

        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            let mut state = app_state.lock().unwrap();
            if event.state == HotKeyState::Released {
                if event.id == *id1.lock().unwrap() {
                    state.start(state.screen_index, state.share_mode); // Avvia la registrazione solo se siamo nella schermata di condivisione
                } else if event.id == *id2.lock().unwrap() {
                    state.stop(); // Ferma la registrazione
                } else if event.id == *id3.lock().unwrap() {
                    state.clear();
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn windows_event_loop(id1: Arc<Mutex<u32>>, id2: Arc<Mutex<u32>>, id3: Arc<Mutex<u32>>, app_state: Arc<Mutex<AppState>>, running: Arc<Mutex<bool>>) {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        loop {
            if !*running.lock().unwrap() {
                break;
            }

            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                let mut state = app_state.lock().unwrap();
                if event.state == HotKeyState::Released {
                    if event.id == *id1.lock().unwrap() {
                        state.start(); // Avvia la registrazione solo se siamo nella schermata di condivisione
                    } else if event.id == *id2.lock().unwrap() {
                        state.stop(); // Ferma la registrazione
                    } else if event.id == *id3.lock().unwrap() {
                        state.clear();
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

