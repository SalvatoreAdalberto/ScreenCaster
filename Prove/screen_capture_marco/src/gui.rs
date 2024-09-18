use iced::widget::{text_input, Button, Column, Container, Row, Text, TextInput, PickList};
use iced::{Alignment, Element, Length, Application, Command, Settings, Theme};
use crate::screen_capture::{ScreenCapture};
use std::sync::{Arc, Mutex};
use crate::AppState;
use iced::keyboard::{KeyCode, Modifiers};

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
    KeyPressChanged(KeyCode), // Nuovo messaggio per modificare la shortcut
    ModifierChanged(Modifiers),
    SaveShortcuts,            // Messaggio per salvare le shortcut
}

// Stati possibili dell'applicazione
#[derive(Debug, Clone, Copy)]
pub enum AppStateEnum {
    Home,
    Sharing,
    Viewing,
}

// Struttura per memorizzare le shortcut personalizzate
#[derive(Debug, Clone)]
pub struct Shortcut {
    pub key: KeyCode,
    pub modifier: Modifiers,
}

// Struttura dell'applicazione
pub struct ScreenCaster {
    state: AppStateEnum,
    ip_address: String,
    ip_input_state: text_input::State,
    screen_capture: ScreenCapture,
    app_state: Arc<Mutex<AppState>>, // Stato condiviso dell'applicazione
    start_shortcut: Shortcut,        // Shortcut per avviare la registrazione
    stop_shortcut: Shortcut,         // Shortcut per fermare la registrazione
    key_list: Vec<KeyCode>,          // Lista di tasti disponibili
    modifier_list: Vec<Modifiers>,   // Lista di modificatori disponibili
}

impl Application for ScreenCaster {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = Arc<Mutex<AppState>>;

    fn new(app_state: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            ScreenCaster {
                state: AppStateEnum::Home,
                ip_address: String::new(),
                ip_input_state: text_input::State::new(),
                screen_capture: ScreenCapture::new(),
                app_state,
                start_shortcut: Shortcut {
                    key: KeyCode::H,
                    modifier: Modifiers::CTRL,
                },
                stop_shortcut: Shortcut {
                    key: KeyCode::F,
                    modifier: Modifiers::CTRL,
                },
                key_list: vec![
                    KeyCode::A, KeyCode::B, KeyCode::C, KeyCode::H, KeyCode::F, // Aggiungi tutti i tasti che ti servono
                ],
                modifier_list: vec![Modifiers::CTRL, Modifiers::SHIFT, Modifiers::LOGO],
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
            Message::KeyPressChanged(key_code) => {
                self.start_shortcut.key = key_code; // Aggiorna la shortcut di avvio
            }
            Message::ModifierChanged(modifier) => {
                self.start_shortcut.modifier = modifier; // Aggiorna il modificatore
            }
            Message::SaveShortcuts => {
                println!(
                    "Shortcut salvata: {:?} + {:?}",
                    self.start_shortcut.modifier, self.start_shortcut.key
                );
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self.state {
            AppStateEnum::Home => self.view_home(),
            AppStateEnum::Sharing => self.view_sharing(),
            AppStateEnum::Viewing => self.view_viewing(),
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
}

pub fn run_gui(app_state: Arc<Mutex<AppState>>) {
    ScreenCaster::run(Settings::with_flags(app_state)).expect("Failed to start application");
}