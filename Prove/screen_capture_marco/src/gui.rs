use iced::widget::{text_input, Button, Column, Container, Row, Text, TextInput};
use iced::{Alignment, Element, Length, Sandbox, Settings, Theme};
use crate::screen_capture::{ScreenCapture, CropArea};

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
}

// Stati possibili dell'applicazione
#[derive(Debug, Clone, Copy)]
pub enum AppState {
    Home,
    Sharing,
    Viewing,
}

// Struttura dell'applicazione
pub struct ScreenCaster {
    state: AppState,
    ip_address: String,
    ip_input_state: text_input::State,
    screen_capture: ScreenCapture,  // Aggiunto per la gestione della registrazione dello schermo
}

impl Sandbox for ScreenCaster {
    type Message = Message;

    fn new() -> Self {
        ScreenCaster {
            state: AppState::Home,
            ip_address: String::new(),
            ip_input_state: text_input::State::new(),
            screen_capture: ScreenCapture::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Screen Casting App")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::GoToShareScreen => {
                self.state = AppState::Sharing;
            }
            Message::GoToViewScreen => {
                self.state = AppState::Viewing;
            }
            Message::StartCasting => {
                self.screen_capture.start();  // Avvia la registrazione
                println!("Screen casting avviato!");
            }
            Message::StopCasting => {
                self.screen_capture.stop();  // Ferma la registrazione
                println!("Screen casting fermato!");
            }
            Message::GoBackHome => {
                self.state = AppState::Home;
            }
            Message::IpAddressChanged(new_ip) => {
                self.ip_address = new_ip;
            }
            Message::SaveIpAddress => {
                println!("Indirizzo IP salvato: {}", self.ip_address);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match self.state {
            AppState::Home => self.view_home(),
            AppState::Sharing => self.view_sharing(),
            AppState::Viewing => self.view_viewing(),
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark  // Aggiunta di un tema scuro
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
                    .push(Button::new(Text::new("Condividi Schermo"))
                              .padding(10)
                              .on_press(Message::GoToShareScreen),)
                    .push(
                        Button::new(Text::new("Guarda Schermo Condiviso"))
                            .padding(10)
                            .on_press(Message::GoToViewScreen),
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
                    .push(Button::new(Text::new("Avvia Screen Casting"))
                              .padding(10)
                              .on_press(Message::StartCasting),)
                    .push(
                        Button::new(Text::new("Ferma Screen Casting"))
                            .padding(10)
                            .on_press(Message::StopCasting),
                    )
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
                    .push(Button::new(Text::new("Salva Indirizzo IP"))
                              .padding(10)
                              .width(Length::Fixed(200.0))
                              .on_press(Message::SaveIpAddress),)
                    .push(
                        Button::new(Text::new("Torna alla Home"))
                            .padding(10)
                            .width(Length::Fixed(200.0))
                            .on_press(Message::GoBackHome),
                    )
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

pub fn run_gui() {
    ScreenCaster::run(Settings::default()).expect("Failed to start application");
}
