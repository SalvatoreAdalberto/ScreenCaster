use iced::widget::{ Button, Column, Container, Row, Text, TextInput, Scrollable, image};
use iced::{Alignment, Element, Length, Application, Command, Settings, Theme, Subscription, window::Action};
use crate::screen_capture::{ScreenCapture};
use crate::utils;
use std::sync::{Arc, Mutex};
use egui::Pos2;
use global_hotkey::GlobalHotKeyManager;
use global_hotkey::hotkey::{HotKey, Modifiers};
use crate::hotkeys::{AppState, parse_key_code};
use std::process::{Child, Command as Command2, Output};
use std::collections::HashMap;
use crate::gif_widget::{self, GifPlayer, GifPlayerMessage};
use crate::streaming_client::{StreamingClient, VideoPlayerMessage};
use iced::window::Event;

// Definiamo i messaggi dell'applicazione
#[derive(Debug, Clone)]
pub enum Message {
    GoToShareScreen,
    GoToViewScreen,
    StartCasting,
    StopCasting,
    GoBackHome,
    SuggestionClicked((String, String)),
    InputChanged(String),
    StartRecordHotkeyChanged(String),
    StopRecordHotkeyChanged(String),
    GoToChangeHotKeys,
    SaveHotKeys,
    ToggleAnnotationMode,
    TryConnect,
    Connecting,
    NoMatchFound,
    MultipleMatches,
    NotInLan,
    VideoPlayerMessage(VideoPlayerMessage),
    StopConnection,
    EventOccurred(Event)
}

// Stati possibili dell'applicazione
#[derive(Debug, Clone, Copy)]
pub enum AppStateEnum {
    Home,
    Sharing,
    Connect,
    ChangeHotKeys,
    Watching,

}

// Struttura dell'applicazione
pub struct ScreenCaster {
    state: AppStateEnum,
    ip_address: String,
    input_state: String,
    screen_capture: ScreenCapture,
    app_state: Arc<Mutex<AppState>>, // Stato condiviso dell'applicazione
    manager: Arc<Mutex<GlobalHotKeyManager>>,
    start_hotkey: HotKey,
    stop_hotkey: HotKey,
    start_id: Arc<Mutex<u32>>,
    stop_id: Arc<Mutex<u32>>,
    start_shortcut: String,        // Shortcut per avviare la registrazione
    stop_shortcut: String,         // Shortcut per fermare la registrazione
    handle_annotation_tool: Option<Child>,
    client: Option<Output>,
    streamers_map: HashMap<String, String>,
    streamers_suggestions: Vec<(String, String)>,
    streaming_client: Option<StreamingClient>,
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
                input_state: String::new(),
                screen_capture: ScreenCapture::new(),
                app_state: flags.0,
                manager: flags.1,
                start_hotkey: flags.4,
                stop_hotkey: flags.5,
                start_id: flags.2,
                stop_id: flags.3,
                start_shortcut: "H".to_string(),
                stop_shortcut: "F".to_string(),
                handle_annotation_tool: None,
                client: None,
                streamers_map: utils::get_streamers_map(),
                streamers_suggestions: Vec::new(),
                streaming_client: None,
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
                self.state = AppStateEnum::Connect;
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
            Message::SuggestionClicked((suggestion, ip)) => {
                self.ip_address = ip;
                self.input_state = suggestion;
                self.streamers_suggestions.clear();

            }
            Message::InputChanged(value) => {
                self.input_state = (&value).to_string();
                self.streamers_suggestions = self
                    .streamers_map
                    .iter()
                    .filter(|(key, ip)| key.to_lowercase().starts_with(&value) || ip.starts_with(&value))
                    .map(|(key, ip)| (key.clone(), ip.clone()))
                    .collect();
                self.ip_address.clear();

            }
            Message::TryConnect => {
                if self.ip_address.is_empty(){
                    let matching = self.streamers_map.iter()
                    .filter(|(key, ip)| key.to_lowercase().starts_with(&self.input_state) || ip.starts_with(&self.input_state))
                    .map(|(_, ip)| ip.clone())
                    .collect::<Vec<String>>();
                    
                    match matching.len() {
                        0 => {
                            return Command::perform(async {}, |_| Message::NoMatchFound);
                        }
                        1 => {
                            self.ip_address = matching[0].clone();
                        }
                        _ => {
                            return Command::perform(async {}, |_| Message::MultipleMatches);
                        }
                    }
                   
                    
                }
                if utils::is_ip_in_lan(&self.ip_address) {
                    self.state = AppStateEnum::Watching;
                    self.streaming_client = Some(StreamingClient::new(self.ip_address.clone()));
                    return Command::perform(async {}, |_| Message::Connecting);
                }else{
                    return Command::perform(async {}, |_| Message::NotInLan );
                }
            }
            Message::VideoPlayerMessage(message) => {
                if let Some(sc) = &mut self.streaming_client {
                    sc.update(message);
                }
            }
            Message::StopConnection => {
                if let Some(sc) = &mut self.streaming_client {
                    sc.update(VideoPlayerMessage::Exit);
                    self.streaming_client = None;
                }
                self.state = AppStateEnum::Connect;
                
            }
            Message::Connecting => {
                if let Some(sc) = &mut self.streaming_client {
                    sc.update(VideoPlayerMessage::Connect);
                }
            }
            Message::NoMatchFound => {
                println!("Nessuna corrispondenza trovata");
            }
            Message::MultipleMatches => {
                println!("Trovate più corrispondenze");
            }
            Message::NotInLan => {
                println!("L'indirizzo IP non è nella LAN");
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
            Message::ToggleAnnotationMode => {
                if self.handle_annotation_tool.is_some() {
                    self.handle_annotation_tool.as_mut().unwrap().kill().unwrap();
                    self.handle_annotation_tool = None;
                } else {
                    self.handle_annotation_tool = Some(Command2::new("./target/debug/annotation_tool")
                        .spawn()
                        .expect("Non è stato possibile avviare la finestra 2"));
                }
            }
            Message::EventOccurred(event) => {
                if let Event::CloseRequested = event{
                    if let Some(_) = self.streaming_client{
                        return Command::perform(async {}, |_| Message::StopConnection);
                    }
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self.state {
            AppStateEnum::Home => self.view_home(),
            AppStateEnum::Sharing => self.view_sharing(),
            AppStateEnum::Connect => self.view_connect(),
            AppStateEnum::ChangeHotKeys => self.view_change_hotkey(),
            AppStateEnum::Watching => self.view_watching(),
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark  // Tema scuro
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.state {
            AppStateEnum::Watching => {if let Some(sc) = self.streaming_client.as_ref() { sc.subscription().map(Message::VideoPlayerMessage)}
                else{ Subscription::none()}},
            _ => {Subscription::none()}
        }
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
                    )
                    .push(
                        Button::new(Text::new("Attiva modalità annotazione"))
                            .padding(10)
                            .on_press(Message::ToggleAnnotationMode),
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
    fn view_connect(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(Text::new("Stai guardando uno schermo condiviso").size(30))
            .push(
                TextInput::new(
                    "Inserisci l'indirizzo IP...",
                    &self.input_state,
                )
                    .padding(10)
                    .width(Length::Fixed(500.0))
                    .on_input(|input| Message::InputChanged(input)),
            )
            .push(
                Scrollable::new(
                        self.streamers_suggestions.iter().fold(Column::new().spacing(5), |column, (suggestion, ip)| {
                            column.push(
                                Button::new(
                                        Row::new()
                                            .spacing(350)
                                            .align_items(Alignment::Center)
                                            .push(Text::new(suggestion))
                                            .push(Text::new(ip))
                                )
                                .on_press(Message::SuggestionClicked((suggestion.clone(), ip.clone())))
                                .padding(8)
                                .width(Length::Fixed(500.0)),
                            )
                        }),
                    ),
                    
            )
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Button::new(Text::new("Connetti"))
                            .padding(10)
                            .width(Length::Fixed(200.0))
                            .on_press(Message::TryConnect),
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

    fn view_watching(&self) -> Element<Message> {
        let content;
        if let Some(sc) = self.streaming_client.as_ref(){
            let video_player = sc.view_video().map(Message::VideoPlayerMessage);
            let record_button = sc.view_record_button().map(Message::VideoPlayerMessage);
            content = Column::new()
                        .push(video_player)
                        .push(
                            Row::new()
                            .spacing(20)
                            .align_items(Alignment::Center)
                            .push(
                                Button::new(Text::new("Stop watching"))
                                    .padding(10)
                                    .width(Length::Fixed(200.0))
                                    .on_press(Message::StopConnection),
                            )
                            .push(
                                record_button
                            ),
                        );
                        

        }else{
            content = Column::new()
                .push(Text::new("SOMETHING WENT WRONG"));
        }
                    
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