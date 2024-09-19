use iced::widget::{text_input, Button, Column, Container, Row, Text, TextInput};
use iced::{Alignment, Element, Length, Application, Command, Settings, Theme};
use crate::screen_capture::{ScreenCapture};
use std::sync::{Arc, Mutex};
use global_hotkey::GlobalHotKeyManager;
use global_hotkey::hotkey::{HotKey, Modifiers};
use crate::hotkeys::{AppState, parse_key_code};

// Definiamo i messaggi dell'applicazione
#[derive(Debug, Clone)]
pub enum Message {
    GoToShareScreen,
    GoToViewScreen,
    StartCasting,
    StopCasting,
    GoBackHome,
    IpAddressChanged(String),
    SaveIpAddress,
    StartRecordHotkeyChanged(String),
    StopRecordHotkeyChanged(String),
    GoToChangeHotKeys,
    SaveHotKeys,
}

// Stati possibili dell'applicazione
#[derive(Debug, Clone, Copy)]
pub enum AppStateEnum {
    Home,
    Sharing,
    Viewing,
    ChangeHotKeys,
}

// Struttura dell'applicazione
pub struct ScreenCaster {
    state: AppStateEnum,
    ip_address: String,
    ip_input_state: text_input::State,
    screen_capture: ScreenCapture,
    app_state: Arc<Mutex<AppState>>, // Stato condiviso dell'applicazione
    manager: Arc<Mutex<GlobalHotKeyManager>>,
    start_hotkey: HotKey,
    stop_hotkey: HotKey,
    start_id: Arc<Mutex<u32>>,
    stop_id: Arc<Mutex<u32>>,
    start_shortcut: String,        // Shortcut per avviare la registrazione
    stop_shortcut: String,         // Shortcut per fermare la registrazione
}

impl Application for ScreenCaster {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = (Arc<Mutex<AppState>>, Arc<Mutex<GlobalHotKeyManager>>, Arc<Mutex<u32>>, Arc<Mutex<u32>>, HotKey, HotKey);

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            ScreenCaster {
                state: AppStateEnum::Home,
                ip_address: String::new(),
                ip_input_state: text_input::State::new(),
                screen_capture: ScreenCapture::new(),
                app_state: flags.0,
                manager: flags.1,
                start_hotkey: flags.4,
                stop_hotkey: flags.5,
                start_id: flags.2,
                stop_id: flags.3,
                start_shortcut: "H".to_string(),
                stop_shortcut: "F".to_string(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Screen Casting App")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        let mut app_state = self.app_state.lock().unwrap();

        match message {
            Message::GoToShareScreen => {
                self.state = AppStateEnum::Sharing;
                app_state.is_sharing = true; // Imposta lo stato di condivisione
            }
            Message::GoToViewScreen => {
                self.state = AppStateEnum::Viewing;
                app_state.is_sharing = false; // Non siamo in condivisione
            }
            Message::StartCasting => {
                app_state.screen_capture.start(); // Avvia la registrazione
                println!("Screen casting avviato!");
            }
            Message::StopCasting => {
                app_state.screen_capture.stop(); // Ferma la registrazione
                println!("Screen casting fermato!");
            }
            Message::GoBackHome => {
                self.state = AppStateEnum::Home;
                app_state.is_sharing = false; // Uscita dalla condivisione
            }
            Message::IpAddressChanged(new_ip) => {
                self.ip_address = new_ip;
            }
            Message::SaveIpAddress => {
                println!("Indirizzo IP salvato: {}", self.ip_address);
            }
            Message::GoToChangeHotKeys => {
                self.state = AppStateEnum::ChangeHotKeys
            }
            Message::SaveHotKeys => {
                let manager = self.manager.lock().unwrap();

                manager.unregister_all(&[self.start_hotkey, self.stop_hotkey]).unwrap();

                let start_code = parse_key_code(&self.start_shortcut).unwrap();
                let stop_code = parse_key_code(&self.stop_shortcut).unwrap();

                #[cfg(target_os = "macos")]
                let hotkey_record = HotKey::new(Some(Modifiers::SUPER), start_code);
                #[cfg(not(target_os = "macos"))]
                let hotkey_record = HotKey::new(Some(Modifiers::CONTROL), start_code);

                #[cfg(target_os = "macos")]
                let hotkey_stop = HotKey::new(Some(Modifiers::SUPER), stop_code);
                #[cfg(not(target_os = "macos"))]
                let hotkey_stop = HotKey::new(Some(Modifiers::CONTROL), stop_code);

                let _ = manager.register(hotkey_record).unwrap();
                let _ = manager.register(hotkey_stop).unwrap();

                self.start_hotkey = hotkey_record;
                self.stop_hotkey = hotkey_stop;

                let mut id1 = self.start_id.lock().unwrap();
                *id1 = hotkey_record.id();
                let mut id2 = self.stop_id.lock().unwrap();
                *id2 = hotkey_stop.id();

                println!("Hotkeys modificate!");
                println!("Start: {}", self.start_shortcut);
                println!("Stop: {}", self.stop_shortcut);
            }
            Message::StartRecordHotkeyChanged(key) => {
                self.start_shortcut = key
            }
            Message::StopRecordHotkeyChanged(key) => {
                self.stop_shortcut = key
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self.state {
            AppStateEnum::Home => self.view_home(),
            AppStateEnum::Sharing => self.view_sharing(),
            AppStateEnum::Viewing => self.view_viewing(),
            AppStateEnum::ChangeHotKeys => self.view_change_hotkey(),
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark  // Tema scuro
    }
}

impl ScreenCaster {
    // Vista della Home Page
    fn view_home(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(Text::new("Benvenuto nell'app di Screen Casting").size(30))
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Button::new(Text::new("Condividi Schermo"))
                            .padding(10)
                            .on_press(Message::GoToShareScreen),
                    )
                    .push(
                        Button::new(Text::new("Guarda Schermo Condiviso"))
                            .padding(10)
                            .on_press(Message::GoToViewScreen),
                    ),
            )
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Button::new(Text::new("Modifica hotkeys"))
                            .padding(10)
                            .on_press(Message::GoToChangeHotKeys),
                    )
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    // Vista per la condivisione dello schermo
    fn view_sharing(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(Text::new("Stai condividendo il tuo schermo").size(30))
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Button::new(Text::new("Avvia Screen Casting"))
                            .padding(10)
                            .on_press(Message::StartCasting),
                    )
                    .push(
                        Button::new(Text::new("Ferma Screen Casting"))
                            .padding(10)
                            .on_press(Message::StopCasting),
                    ),
            )
            .push(
                Button::new(Text::new("Torna alla Home"))
                    .padding(10)
                    .on_press(Message::GoBackHome),
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    // Vista per la visualizzazione dello schermo condiviso
    fn view_viewing(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(Text::new("Stai guardando uno schermo condiviso").size(30))
            .push(
                TextInput::new(
                    "Inserisci l'indirizzo IP...",
                    &self.ip_address,
                )
                    .padding(10)
                    .width(Length::Fixed(300.0))
                    .on_input(Message::IpAddressChanged),
            )
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Button::new(Text::new("Salva Indirizzo IP"))
                            .padding(10)
                            .width(Length::Fixed(200.0))
                            .on_press(Message::SaveIpAddress),
                    )
                    .push(
                        Button::new(Text::new("Torna alla Home"))
                            .padding(10)
                            .width(Length::Fixed(200.0))
                            .on_press(Message::GoBackHome),
                    ),
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn view_change_hotkey(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(Text::new("Configurazione hotkey").size(30))
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Text::new("Inserisci la key per iniziare la registrazione:").size(20)
                    )
                    .push(
                        TextInput::new(
                            "Inserisci l'hotkey per iniziare la registrazione",
                            &self.start_shortcut.to_uppercase(),
                        )
                            .padding(10)
                            .width(Length::Fixed(50.0))
                            .on_input(Message::StartRecordHotkeyChanged),
                    )
            )
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Text::new("Inserisci la key per iniziare la registrazione:").size(20)
                    )
                    .push(
                        TextInput::new(
                            "Inserisci l'hotkey per fermatre la registrazione",
                            &self.stop_shortcut.to_uppercase(),
                        )
                            .padding(10)
                            .width(Length::Fixed(50.0))
                            .on_input(Message::StopRecordHotkeyChanged),
                    )
            )
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Button::new(Text::new("Salva hotkeys"))
                            .padding(10)
                            .width(Length::Fixed(200.0))
                            .on_press(Message::SaveHotKeys),
                    )
                    .push(
                        Button::new(Text::new("Torna alla Home"))
                            .padding(10)
                            .width(Length::Fixed(200.0))
                            .on_press(Message::GoBackHome),
                    ),
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

pub fn run_gui(app_state: Arc<Mutex<AppState>>, manager: Arc<Mutex<GlobalHotKeyManager>>, id1: Arc<Mutex<u32>>, id2: Arc<Mutex<u32>>, hotkey_record: HotKey, hotkey_stop: HotKey) {
    ScreenCaster::run(Settings::with_flags((app_state, manager, id1, id2, hotkey_record, hotkey_stop))).expect("Failed to start application");
}