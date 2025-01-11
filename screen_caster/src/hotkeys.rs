use std::io::Write;
use global_hotkey::{GlobalHotKeyEvent, HotKeyState, hotkey::Code};
use std::sync::{Arc, Mutex};
use std::time::Duration;
#[cfg(target_os = "windows")]
use winapi::um::winuser::{self, MSG};
#[cfg(target_os = "windows")]
use winapi::shared::winerror::WAIT_TIMEOUT;
#[cfg(target_os = "windows")]
use winapi::um::winbase::WAIT_OBJECT_0;
use crate::streaming_server;
use crate::gui::ShareMode;
use iced::{ Subscription, time as iced_time};

/// This module handles the global hotkeys for the application.
/// The hotkeys are used to start and stop the screen sharing, to clear the annotations and to end a session.


/// Enumerates the possible messages that can be sent by the hotkeys.
/// - Start: message to start the screen sharing
/// - Stop: message to stop the screen sharing
/// - CloseSessionServer: message to close the session on the server side
/// - CloseSessionClient: message to close the session on the client side
/// The messages are used to communicate with the main application.
#[derive(Debug, Clone)]
pub enum HotkeyMessage {
    Start,
    Stop,
    CloseSessionServer,
    CloseSessionClient,
}

pub struct AppState {
    // Flag to check if the application is in casting mode, readay to start the screen sharing.
    pub(crate) is_sharing: bool,
    // The streaming server used to record the screen.
    pub(crate) streaming_server: Option<streaming_server::StreamingServer>,
    // The mode of sharing (Fullscreen or Area). Default is Fullscreen.
    pub(crate) share_mode: ShareMode,
    // The index of the screen to share. Default is 1 (the main screen).
    pub(crate) screen_index: usize, 
    // The stdin of the annotation tool process to send the clear command.
    pub(crate) annotation_stdin: Option<std::process::ChildStdin>,
    // Flag to check if the screen sharing has started.
    pub(crate) cast_started: bool,
    // Flag to check if the session has been closed.
    pub(crate) session_closed: bool,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            is_sharing: false,
            streaming_server: None,
            share_mode: ShareMode::Fullscreen,
            screen_index: 1,
            annotation_stdin: None,
            cast_started: false,
            session_closed: false,
        }
    }

    // Start the StreamingServer with the selected screen index and share mode.
    pub fn start(&mut self) {
        if self.is_sharing && !self.cast_started {
            self.session_closed = false;
            if self.streaming_server.is_none(){
                self.streaming_server = Some(streaming_server::StreamingServer::new());
            }
            self.streaming_server.as_mut().unwrap().start(self.screen_index, self.share_mode);
            self.cast_started = true;
        }
    }

    // Stop the StreamingServer and close the annotation tool.
    pub fn stop(&mut self) {
        if self.is_sharing && self.cast_started {
            if self.streaming_server.is_some(){
                self.streaming_server.as_mut().unwrap().stop();
                self.streaming_server = None;
            }
            self.close_annotation();
            self.cast_started = false;
        }
    }

    // Send a clear command to the annotation tool to delete all the shapes.
    pub fn clear(&mut self) {
        if let Some(ref mut std) = self.annotation_stdin {
            if writeln!(std, "clear").is_ok() {
                // Do nothing, the annotation tool will clear the shapes.
            } else {
                self.annotation_stdin = None;
            }
        }
    }

    // Update the stdin of the annotation tool process in case it is closed and reopened.
    pub fn update_stdin(&mut self, stdin: std::process::ChildStdin) {
        // Double check if the annotation tool is still open.
        self.close_annotation();
        self.annotation_stdin = Some(stdin);
    }

    // Close the stdin of the annotation tool process.
    pub fn close_annotation(&mut self) {
        if let Some(ref mut std) = self.annotation_stdin {
            // Send the quit command to the annotation tool.
            if writeln!(std, "quit").is_ok() {
                self.annotation_stdin = None;
            } else {
                // The annotation tool is already closed.
                self.annotation_stdin = None;
            }
        }
    }

    // Check if the annotation tool is still open to avoid opening multiple instances.
    pub fn check_annotation_open(&mut self) -> bool {
        if let Some(ref mut std) = self.annotation_stdin {
            // Check if the stdin is still open.
            if writeln!(std, "check").is_ok() {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    // Stop the session. If used by the server, it will close the StreamingServer and the annotation tool.
    pub fn stop_session(&mut self) {
        self.session_closed = true;
        if self.cast_started{
            self.stop();
        }
    }

    /// Subscription to send hotkey messages to the main application.
    /// It allows to update the gui and the application state.
    pub fn subscription(&mut self) -> Subscription<HotkeyMessage> {
        if self.cast_started && self.is_sharing {
            iced_time::every(Duration::from_secs_f32(0.1))
                .map(|_| HotkeyMessage::Start)
        } else if self.is_sharing && self.session_closed {
            iced_time::every(Duration::from_secs_f32(0.1))
                .map(|_| HotkeyMessage::CloseSessionServer)
        }
        else if !self.cast_started && self.is_sharing && !self.session_closed {
            iced_time::every(Duration::from_secs_f32(0.1))
                .map(|_| HotkeyMessage::Stop)
        }  else if !self.is_sharing && self.session_closed {
            iced_time::every(Duration::from_secs_f32(0.1))
                .map(|_| HotkeyMessage::CloseSessionClient)
        }
        else {
            Subscription::none()
        }
    }
}

// Event loop for the hotkeys on MacOS.
#[cfg(target_os = "macos")]
pub fn macos_event_loop(id1: Arc<Mutex<u32>>, id2: Arc<Mutex<u32>>, id3: Arc<Mutex<u32>>, id4: Arc<Mutex<u32>>, app_state: Arc<Mutex<AppState>>, running: Arc<Mutex<bool>>) {
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
                } else if event.id == *id4.lock().unwrap() {
                    state.stop_session();
                }
            }
        }
    }
}

// Event loop for the hotkeys on Linux.
#[cfg(target_os = "linux")]
pub fn linux_event_loop(id1: Arc<Mutex<u32>>, id2: Arc<Mutex<u32>>, id3: Arc<Mutex<u32>>, id4: Arc<Mutex<u32>>, app_state: Arc<Mutex<AppState>>, running: Arc<Mutex<bool>>) {
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
                } else if event.id == *id4.lock().unwrap() {
                    state.stop_session();
                }
            }
        }
    }
}

// Event loop for the hotkeys on Windows.
#[cfg(target_os = "windows")]
pub fn windows_event_loop(id1: Arc<Mutex<u32>>, id2: Arc<Mutex<u32>>, id3: Arc<Mutex<u32>>, id4: Arc<Mutex<u32>>, app_state: Arc<Mutex<AppState>>, running: Arc<Mutex<bool>>) {
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
                    } else if event.id == *id4.lock().unwrap() {
                        state.stop_session();
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

// Parse the key code to the corresponding hotkey code.
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
