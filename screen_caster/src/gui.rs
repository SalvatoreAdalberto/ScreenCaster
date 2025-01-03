use ffmpeg_sidecar::command;
use iced::widget::{Button, Column, Container, Row, Text, TextInput, Scrollable, PickList, Space};
use iced::{Alignment, Element, Length, Application, Command, Settings, Theme, Subscription, alignment::Horizontal};
use crate::utils;
use std::sync::{Arc, Mutex};
use global_hotkey::GlobalHotKeyManager;
use global_hotkey::hotkey::{HotKey, Modifiers};
use crate::hotkeys::{AppState, parse_key_code, HotkeyMessage};
use std::process::{Command as Command2, Output, Stdio};
use std::collections::HashMap;
use crate::streaming_client::{StreamingClient, VideoPlayerMessage};
use crate::streamers_table::{StreamersTable, StreamersTableMessage};
use iced::Event;
use native_dialog::FileDialog;

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
    ClearHotkeyChanged(String),
    CloseHotkeyChanged(String),
    GoToChangeHotKeys,
    GoToSettings,
    GoToChangeDirectory,
    SaveHotKeys,
    ToggleAnnotationTool,
    SelectCropArea,
    TryConnect,
    Connecting,
    NoMatchFound,
    MultipleMatches,
    NotInLan,
    VideoPlayerMessage(VideoPlayerMessage),
    StopConnection,
    PickList(usize),
    ScreenSelected,
    ModeSelected(ShareMode),
    HotkeyMessage(HotkeyMessage),
    BrowseDirectory,
    DirectorySelected(Option<String>),
    SaveDirectory,
    GoToListStreamers,
    StreamersTableMessage(StreamersTableMessage)
}

// Stati possibili dell'applicazione
#[derive(Debug, Clone, Copy)]
pub enum AppStateEnum {
    Home,
    StartSharing,
    IsSharing,
    Connect,
    ChangeHotKeys,
    ChangeDirectory,
    Settings,
    Watching,
    SelectScreen,
    ChangeListStreamers,
}

// Struttura dell'applicazione
pub struct ScreenCaster {
    state: AppStateEnum,
    ip_address: String,
    input_state: String,
    app_state: Arc<Mutex<AppState>>, // Stato condiviso dell'applicazione
    manager: Arc<Mutex<GlobalHotKeyManager>>,
    start_hotkey: HotKey,
    stop_hotkey: HotKey,
    clear_hotkey: HotKey,
    close_hotkey: HotKey,
    start_id: Arc<Mutex<u32>>,
    stop_id: Arc<Mutex<u32>>,
    clear_id: Arc<Mutex<u32>>,
    close_id: Arc<Mutex<u32>>,
    start_shortcut: String,        // Shortcut per avviare la registrazione
    stop_shortcut: String,         // Shortcut per fermare la registrazione
    clear_shortcut: String,
    close_shortcut: String,
    client: Option<Output>,
    streamers_table: StreamersTable,
    streamers_map: HashMap<String,String>,
    streamers_suggestions: Vec<(String, String)>,
    streaming_client: Option<StreamingClient>,
    screen_index: usize,
    share_mode: ShareMode,
    selected_directory: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareMode {
    Fullscreen,
    CropArea,
}

impl std::fmt::Display for ShareMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShareMode::Fullscreen => write!(f, "Fullscreen"),
            ShareMode::CropArea => write!(f, "Crop Area"),
        }
    }
}

impl Application for ScreenCaster {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = (Arc<Mutex<AppState>>, Arc<Mutex<GlobalHotKeyManager>>, Arc<Mutex<u32>>, Arc<Mutex<u32>>, Arc<Mutex<u32>>, Arc<Mutex<u32>>, HotKey, HotKey, HotKey, HotKey);

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (start, stop, clear, close) = utils::read_hotkeys().unwrap();
        let save_path = utils::get_save_directory().unwrap();
        (
            ScreenCaster {
                state: AppStateEnum::Home,
                ip_address: String::new(),
                input_state: String::new(),
                app_state: flags.0,
                manager: flags.1,
                start_hotkey: flags.6,
                stop_hotkey: flags.7,
                clear_hotkey: flags.8,
                close_hotkey: flags.9,
                start_id: flags.2,
                stop_id: flags.3,
                clear_id: flags.4,
                close_id: flags.5,
                start_shortcut: start,
                stop_shortcut: stop,
                clear_shortcut: clear,
                close_shortcut: close,
                client: None,
                streamers_table: StreamersTable::new(),
                streamers_map: HashMap::new(),
                streamers_suggestions: Vec::new(),
                streaming_client: None,
                screen_index: 1,
                share_mode: ShareMode::Fullscreen,
                selected_directory: save_path,
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
                self.state = AppStateEnum::SelectScreen;
            }
            Message::GoToViewScreen => {
                self.state = AppStateEnum::Connect;
                self.streamers_map = HashMap::new();
                self.streamers_table.get_users().into_iter().for_each(|(_, (name, ip))|{
                    self.streamers_map.insert(name, ip);
                });
                app_state.is_sharing = false; // Non siamo in condivisione
            }
            Message::ScreenSelected => {
                if app_state.share_mode == ShareMode::CropArea {
                    let exe_path = utils::get_project_src_path();
                    let mut real_path = "".to_string();
                    real_path = exe_path.display().to_string() + r"/overlay_crop/target/release/overlay_crop";
                    Command2::new(real_path)
                        .arg(app_state.screen_index.to_string())
                        .output()
                        .expect("Non è stato possibile avviare l'overlay crop");
                }
                app_state.is_sharing = true; // Imposta lo stato di condivisione
                self.state = AppStateEnum::StartSharing;
            }
            Message::StartCasting => {
                app_state.start(); // Avvia la registrazione
                self.state = AppStateEnum::IsSharing;
                println!("Screen casting avviato!");
            }
            Message::StopCasting => {
                app_state.stop(); // Ferma la registrazione
                self.state = AppStateEnum::StartSharing;
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
                if self.input_state.is_empty() {
                    self.streamers_suggestions.clear();
                    return Command::none();
                }
                self.streamers_suggestions = self
                    .streamers_map
                    .iter()
                    .filter(|(key, ip)| key.starts_with(&self.input_state.to_ascii_lowercase()) || ip.starts_with(&self.input_state.to_ascii_lowercase()))
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
                            self.ip_address = self.input_state.clone();
                            let _ = Command::perform(async {}, |_| Message::NoMatchFound);
                        }
                        1 => {
                            self.ip_address = matching[0].clone();
                        }
                        _ => {
                            return Command::perform(async {}, |_| Message::MultipleMatches);
                        }
                    }


                }
                return if utils::is_ip_in_lan(&self.ip_address) {
                    self.state = AppStateEnum::Watching;
                    self.streaming_client = Some(StreamingClient::new(self.ip_address.clone(), self.selected_directory.clone()));
                    Command::perform(async {}, |_| Message::Connecting)
                } else {
                    Command::perform(async {}, |_| Message::NotInLan)
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

                manager.unregister_all(&[self.start_hotkey, self.stop_hotkey, self.clear_hotkey, self.close_hotkey]).unwrap();

                let start_code = parse_key_code(&self.start_shortcut).unwrap();
                let stop_code = parse_key_code(&self.stop_shortcut).unwrap();
                let clear_code = parse_key_code(&self.clear_shortcut).unwrap();
                let close_code = parse_key_code(&self.close_shortcut).unwrap();

                #[cfg(target_os = "macos")]
                let hotkey_record = HotKey::new(Some(Modifiers::SUPER), start_code);
                #[cfg(not(target_os = "macos"))]
                let hotkey_record = HotKey::new(Some(Modifiers::CONTROL), start_code);

                #[cfg(target_os = "macos")]
                let hotkey_stop = HotKey::new(Some(Modifiers::SUPER), stop_code);
                #[cfg(not(target_os = "macos"))]
                let hotkey_stop = HotKey::new(Some(Modifiers::CONTROL), stop_code);

                #[cfg(target_os = "macos")]
                let hotkey_clear = HotKey::new(Some(Modifiers::SUPER), clear_code);
                #[cfg(not(target_os = "macos"))]
                let hotkey_clear = HotKey::new(Some(Modifiers::CONTROL), clear_code);

                #[cfg(target_os = "macos")]
                let hotkey_close = HotKey::new(Some(Modifiers::SUPER), close_code);
                #[cfg(not(target_os = "macos"))]
                let hotkey_close = HotKey::new(Some(Modifiers::CONTROL), close_code);

                let _ = manager.register(hotkey_record).unwrap();
                let _ = manager.register(hotkey_stop).unwrap();
                let _ = manager.register(hotkey_clear).unwrap();
                let _ = manager.register(hotkey_close).unwrap();

                self.start_hotkey = hotkey_record;
                self.stop_hotkey = hotkey_stop;
                self.clear_hotkey = hotkey_clear;
                self.close_hotkey = hotkey_close;

                let mut id1 = self.start_id.lock().unwrap();
                *id1 = hotkey_record.id();
                let mut id2 = self.stop_id.lock().unwrap();
                *id2 = hotkey_stop.id();
                let mut id3 = self.clear_id.lock().unwrap();
                *id3 = hotkey_clear.id();
                let mut id4 = self.close_id.lock().unwrap();
                *id4 = hotkey_close.id();

                utils::save_hotkeys(&self.start_shortcut, &self.stop_shortcut, &self.clear_shortcut, &self.close_shortcut).unwrap();

                println!("Hotkeys modificate!");
                println!("Start: {}", self.start_shortcut);
                println!("Stop: {}", self.stop_shortcut);
                println!("Clear: {}", self.clear_shortcut);
                println!("Close: {}", self.close_shortcut);

                self.state = AppStateEnum::Settings;
            }
            Message::StartRecordHotkeyChanged(key) => {
                self.start_shortcut = key
            }
            Message::StopRecordHotkeyChanged(key) => {
                self.stop_shortcut = key
            }
            Message::ClearHotkeyChanged(key) => {
                self.clear_shortcut = key
            }
            Message::CloseHotkeyChanged(key) => {
                self.close_shortcut = key
            }
            Message::ToggleAnnotationTool => {
                 if !app_state.check_annotation_open() {

                    let exe_path = utils::get_project_src_path();
                    let mut real_path = "".to_string();
                    real_path = exe_path.display().to_string() + r"/annotation_tool/target/release/annotation_tool";
                    let child = Some(Command2::new(real_path)
                        .arg(app_state.screen_index.to_string())
                        .stdin(Stdio::piped())
                        .spawn()
                        .expect("Non è stato possibile avviare l'annotation tool"));
                    app_state.update_stdin(child.unwrap().stdin.unwrap());
                } else {
                    println!("Annotation tool già aperto");
                }
            }
            Message::SelectCropArea => {
                let exe_path = utils::get_project_src_path();
                let mut real_path = "".to_string();
                real_path = exe_path.display().to_string() + r"/overlay_crop/target/release/overlay_crop";
                Command2::new(real_path)
                    .arg(app_state.screen_index.to_string())
                    .output()
                    .expect("Non è stato possibile avviare l'overlay crop");
            }
            Message::PickList(n) => {
                app_state.screen_index = n;
                self.screen_index = n;
            }
            Message::ModeSelected(mode) => {
                app_state.share_mode = mode;
                self.share_mode = mode;
            }
            Message::HotkeyMessage(message) => {
                match message {
                    HotkeyMessage::Start => {
                        self.state = AppStateEnum::IsSharing;
                    }
                    HotkeyMessage::Stop => {
                        self.state = AppStateEnum::StartSharing;
                    }
                    HotkeyMessage::CloseSessionServer => {
                        app_state.is_sharing = false;
                        app_state.session_closed = false;
                        self.state = AppStateEnum::Home;
                    }
                    HotkeyMessage::CloseSessionClient => {
                        app_state.session_closed = false;
                        if let Some(sc) = &mut self.streaming_client {
                            sc.update(VideoPlayerMessage::Exit);
                            self.streaming_client = None;
                        }
                        self.state = AppStateEnum::Home;
                    }
                }
            }
            Message::BrowseDirectory => {
                // Apri il file dialog per selezionare una directory
                let selected_directory = FileDialog::new()
                    .show_open_single_dir()
                    .ok()
                    .flatten();
                return Command::perform(async {}, |_| Message::DirectorySelected(selected_directory.map(|d| d.display().to_string())));
            }
            Message::DirectorySelected(directory) => {
                if let Some(directory) = directory {
                    self.selected_directory = directory;
                }
            }
            Message::GoToSettings => {
                self.state = AppStateEnum::Settings;
            }
            Message::GoToChangeDirectory => {
                self.state = AppStateEnum::ChangeDirectory;
            }
            Message::SaveDirectory => {
                let dir = self.selected_directory.clone();
                utils::save_directory(&dir).unwrap();
                println!("Directory salvata: {}", dir);
                self.state = AppStateEnum::Settings;
            }
            Message::GoToListStreamers => {
                self.state = AppStateEnum::ChangeListStreamers;
            }
            Message::StreamersTableMessage(message) => {
                if let StreamersTableMessage::Exit = message {
                    self.streamers_table.update(message);
                    return Command::perform(async {}, |_| Message::GoToViewScreen);
                } else {
                    self.streamers_table.update(message); 
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self.state {
            AppStateEnum::Home => self.view_home(),
            AppStateEnum::StartSharing => self.view_start_sharing(),
            AppStateEnum::IsSharing => self.view_is_sharing(),
            AppStateEnum::Connect => self.view_connect(),
            AppStateEnum::ChangeHotKeys => self.view_change_hotkey(),
            AppStateEnum::Watching => self.view_watching(),
            AppStateEnum::SelectScreen => self.view_select_screen(),
            AppStateEnum::Settings => self.view_settings(),
            AppStateEnum::ChangeDirectory => self.view_save_directory(),
            AppStateEnum::ChangeListStreamers => self.view_modify_list_streamers(),
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark  // Tema scuro
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.state {
            AppStateEnum::Watching => {
                let mut app_state = self.app_state.lock().unwrap();
                if let Some(sc) = self.streaming_client.as_ref() {
                    Subscription::batch(vec![sc.subscription().map(Message::VideoPlayerMessage), app_state.subscription().map(Message::HotkeyMessage)])

                }
                else{
                    Subscription::none()
                }},
            AppStateEnum::IsSharing | AppStateEnum::StartSharing => {
                let mut app_state = self.app_state.lock().unwrap();
                Subscription::batch(vec![app_state.subscription().map(Message::HotkeyMessage)])
            }
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
                        Button::new(Text::new("Impostazioni"))
                            .padding(10)
                            .on_press(Message::GoToSettings),
                    )
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn view_select_screen(&self) -> Element<Message> {
        let n_screens = utils::count_screens();
        let screens = (1..=n_screens).collect::<Vec<usize>>();
        let modes = vec![ShareMode::Fullscreen, ShareMode::CropArea];


        let content = Column::new()
            .spacing(40)
            .align_items(Alignment::Center)
            .push(Text::new("Seleziona lo schermo da condividere").size(30))
            .push(
                Row::new()
                    .spacing(40)
                    .align_items(Alignment::Center)
                    .push(
                        Column::new()
                            .spacing(20)
                            .align_items(Alignment::Center)
                            .push(Text::new("Schermo da condividere:").size(20))
                            .push(
                                PickList::new(
                                    screens,
                                    Some(self.screen_index),
                                    Message::PickList,
                                )
                                    .placeholder("Seleziona uno schermo..."),
                            ),
                    )
                    .push(
                        Column::new()
                            .spacing(20)
                            .align_items(Alignment::Center)
                            .push(Text::new("Modalità di condivisione:").size(20))
                            .push(
                                PickList::new(
                                    modes,
                                    Some(self.share_mode),
                                    Message::ModeSelected,
                                )
                                    .placeholder("Seleziona una modalità..."),
                            ),
                    ),
            )
            .push(Space::with_height(30))
            .push(
                Row::new()
                    .spacing(40)
                    .align_items(Alignment::Center)
                    .push(
                        Button::new(Text::new("Torna alla Home"))
                            .padding(10)
                            .on_press(Message::GoBackHome),
                    )
                    .push(
                        Button::new(Text::new("Conferma"))
                            .padding(10)
                            .on_press(Message::ScreenSelected),
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
    fn view_start_sharing(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(Text::new("Condividi il tuo schermo").size(30))
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Button::new(Text::new("Avvia Screen Casting"))
                            .padding(10)
                            .on_press(Message::StartCasting),
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

    fn view_is_sharing(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(Text::new("Stai condividendo il tuo schermo").size(30))
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Button::new(Text::new("Ferma Screen Casting"))
                            .padding(10)
                            .on_press(Message::StopCasting),
                    )
                    .push(
                        Button::new(Text::new("Annotation tool"))
                            .padding(10)
                            .on_press(Message::ToggleAnnotationTool)
                    ),
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
                                        .push(
                                        Column::new()
                                                .width(Length::Fixed(250.0))
                                                .align_items(Alignment::Start)
                                                .push(Text::new(suggestion))
                                        )
                                        .push(
                                        Column::new()
                                                .width(Length::Fixed(250.0))
                                                .align_items(Alignment::End)
                                                .push(Text::new(ip))
                            ))
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
                        Button::new(Text::new("Gestisci lista streamers"))
                            .padding(10)
                            .width(Length::Fixed(200.0))
                            .on_press(Message::GoToListStreamers),
                    ),
            )
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Button::new(Text::new("Torna alla home"))
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

    fn view_watching(&self) -> Element<Message> {
        let content;
        if let Some(sc) = self.streaming_client.as_ref(){
            let mut row = Row::new()
                        .spacing(20)
                        .padding(10)
                        .align_items(Alignment::Center)
                        .push(
                            Button::new(Text::new("Stop watching"))
                                .padding(10)
                                .width(Length::Fixed(200.0))
                                .on_press(Message::StopConnection),
                        );
                        
            let video_player = sc.view_video().map(Message::VideoPlayerMessage);
            let optional_button = sc.view_record_button();
            if let Some(record_button) = optional_button{
                row = row.push(record_button.map(Message::VideoPlayerMessage));
            }
            content = Column::new()
                .push(video_player)
                .push(
                    row
                )
                .align_items(Alignment::Center);

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

    fn view_settings(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(Text::new("Impostazioni").size(30))
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Button::new(Text::new("Configura hotkeys"))
                            .padding(10)
                            .on_press(Message::GoToChangeHotKeys),
                    )
                    .push(
                        Button::new(Text::new("Configura save directory"))
                            .padding(10)
                            .on_press(Message::GoToChangeDirectory),
                    )
            )
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Button::new(Text::new("Torna alla Home"))
                            .padding(10)
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
                        Text::new("Inserisci la key per fermare la registrazione:").size(20)
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
                        Text::new("Inserisci la key per cancellare le annotazioni:").size(20)
                    )
                    .push(
                        TextInput::new(
                            "Inserisci l'hotkey per cancellare le annotazioni",
                            &self.clear_shortcut.to_uppercase(),
                        )
                            .padding(10)
                            .width(Length::Fixed(50.0))
                            .on_input(Message::ClearHotkeyChanged),
                    )
            )
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Text::new("Inserisci la key per terminare la sessione:").size(20)
                    )
                    .push(
                        TextInput::new(
                            "Inserisci l'hotkey per terminare la sessione",
                            &self.close_shortcut.to_uppercase(),
                        )
                            .padding(10)
                            .width(Length::Fixed(50.0))
                            .on_input(Message::CloseHotkeyChanged),
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
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn view_save_directory(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(Text::new("Configura la directory di salvataggio").size(30))
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Text::new(self.selected_directory.clone())
                    )
                    .push(
                        Button::new(Text::new("Sfoglia"))
                            .on_press(Message::BrowseDirectory),
                    )
            )
            .push(
                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(
                        Button::new(Text::new("Salva modifiche"))
                            .padding(10)
                            .on_press(Message::SaveDirectory),
                    )
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn view_modify_list_streamers(&self) -> Element<Message> {
        
        Container::new(self.streamers_table.view_streamers_table().map(Message::StreamersTableMessage))
            .center_x()
            .center_y()
            .height(Length::Fill)
            .into()
    }

}

pub fn run_gui(app_state: Arc<Mutex<AppState>>, manager: Arc<Mutex<GlobalHotKeyManager>>, id1: Arc<Mutex<u32>>, id2: Arc<Mutex<u32>>, id3: Arc<Mutex<u32>>, id4: Arc<Mutex<u32>>, hotkey_record: HotKey, hotkey_stop: HotKey, hotkey_clear: HotKey, hotkey_close: HotKey) {
    let mut app_state_clone = app_state.clone();
    let mut settings = Settings::with_flags((app_state, manager, id1, id2, id3, id4, hotkey_record, hotkey_stop, hotkey_clear, hotkey_close));
    ScreenCaster::run(settings).expect("Failed to start application");
    app_state_clone.lock().unwrap().stop();
}