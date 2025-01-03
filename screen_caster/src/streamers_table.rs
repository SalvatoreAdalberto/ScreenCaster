use std::collections::HashMap;
use iced::{Element, Alignment, Color, Theme, Length, theme, alignment::Horizontal};
use iced::widget::{Row, Text, TextInput, Button, Column, Container, Svg};
use rusqlite::{params, Connection};
use uuid::Uuid;
use thiserror::Error;
use std::net::Ipv4Addr;
use crate::error_banner::{Banner, InputError};

struct InputErrorBanner;

impl<'a> Banner<'a> for InputErrorBanner{
    type ExtMessage = StreamersTableMessage;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CudEnum{
    Create,
    Update,
    Delete
}

pub struct StreamersTable{
    db_connection: Connection,
    name_input: String,
    ip_input: String,
    ante_error_state: Option<StreamersTableStateEnum>,
    editing_id: Option<String>,
    state: StreamersTableStateEnum

}

#[derive(Debug, Clone, Copy)]
enum StreamersTableStateEnum{
    Instantiated,
    Creating,
    Editing,
    Error(InputError),
}

#[derive(Debug, Clone)]
pub enum StreamersTableMessage{
    InputName(String),
    InputIp(String),
    CheckModifications((Option<String>, CudEnum)),
    CloseBanner,
    Modify(String),
    ToggleAdd,
    Exit,
}

pub const STREAMERS_LIST_PATH : &str = "../config/streamers_list.db";

impl StreamersTable{
    pub fn new() -> Self{
        let db_connection = Connection::open(STREAMERS_LIST_PATH).unwrap();
        db_connection.execute(
            "CREATE TABLE IF NOT EXISTS streamers (
                      id TEXT PRIMARY KEY,
                      name TEXT NOT NULL UNIQUE,
                      ip TEXT NOT NULL UNIQUE
                  )",
            [],
        ).unwrap();

        Self { db_connection ,name_input: "".to_string(), ip_input: "".to_string(), editing_id: None,  state: StreamersTableStateEnum::Instantiated, ante_error_state: None}
    }

    pub fn update(&mut self, message: StreamersTableMessage){
        match message{
            StreamersTableMessage::InputName(name) => {
                self.name_input = name;
            },
            StreamersTableMessage::InputIp(ip) => {
                self.ip_input = ip;
            },
            StreamersTableMessage::CheckModifications((id, cud)) => {
                match self.state {
                    StreamersTableStateEnum::Error(_) => {},
                    StreamersTableStateEnum::Creating | StreamersTableStateEnum::Editing if cud == CudEnum::Delete => {
                    },
                    _ => {
                        match self.check_modifications(id, cud){
                            Ok((id, name, ip)) => {
                                match cud{
                                    CudEnum::Create => {
                                        add_record(&self.db_connection, name, ip);
                                    },
                                    CudEnum::Update => {
                                        modify_record(&self.db_connection, name, ip, id.unwrap());
                                    },
                                    CudEnum::Delete => {
                                        delete_record(&self.db_connection, id.unwrap());
                                    }
                                    
                                }
                                self.ip_input = "".to_string();
                                self.name_input = "".to_string();
                                self.editing_id = None;
                                self.state = StreamersTableStateEnum::Instantiated;
                            },
                            Err(e) => {
                                self.ante_error_state = Some(self.state);
                                self.state = StreamersTableStateEnum::Error(e);
                            }
                        }
                    }
                }
                
            },
            StreamersTableMessage::CloseBanner =>{
                self.state = self.ante_error_state.unwrap();
            }
            StreamersTableMessage::ToggleAdd => {
                match self.state{
                    StreamersTableStateEnum::Instantiated  =>  self.state = StreamersTableStateEnum::Creating,
                    StreamersTableStateEnum::Creating => {
                        self.ip_input = "".to_string();
                        self.name_input = "".to_string();
                        self.state = StreamersTableStateEnum::Instantiated
                    },
                    _ => {}
                }
            },
            StreamersTableMessage::Modify(id) => {
                match self.state{
                    StreamersTableStateEnum::Instantiated  => {
                        self.ip_input = "".to_string();
                        self.name_input = "".to_string();
                        self.state = StreamersTableStateEnum::Editing;
                        self.editing_id = Some(id);
                    },
                    _ => {}
                }
            },
            StreamersTableMessage::Exit => {
                self.ip_input = "".to_string();
                self.name_input = "".to_string();
                self.state = StreamersTableStateEnum::Instantiated;
            }
        }
    }

    pub fn view_streamers_table(&self) -> Element<StreamersTableMessage>{
        let header: Element<StreamersTableMessage> = Container::<StreamersTableMessage>::new(
    Row::new()
                .push(
                    Container::new(Text::new("Name").size(30)).width(Length::FillPortion(2)).center_x()
                )
                .push(
                    Container::new(Text::new("IP Address").size(30)).width(Length::FillPortion(2)).center_x()
                )
                .push(
                    Container::new(Text::new("")).width(Length::FillPortion(1))
                )
                .align_items(Alignment::Center).spacing(50)
            ).style(theme::Container::Custom(RecordStyle.into())).into();

        let streamers = self.get_users();
        let mut records = streamers.into_iter().collect::<Vec<(String, (String, String))>>();
        records.sort_by(|a, b| a.0.cmp(&b.0));
        let rows = records.into_iter().map(|x|{
                self.view_record(x)
            }).collect::<Vec<Element<StreamersTableMessage>>>();
        let mut content = Column::new();
        for row  in rows.into_iter(){
            content = content.push(row);
        }
        let table = Container::new(content).width(Length::Fill).center_x();
        let add_button= Container::new(
            Button::new(Svg::from_path("../assets/plus.svg")).width(Length::Fixed(80.0))
            .height(Length::Fixed(40.0)).on_press(StreamersTableMessage::ToggleAdd)
        ).center_x().width(Length::Fill).padding(20);
        let mut col = Column::new().push(header).push(table).padding(40);
        match self.state{
            StreamersTableStateEnum::Creating =>{
                let new_record_input: Element<StreamersTableMessage> = Container::<StreamersTableMessage>::new(
                    Row::new()
                    .push(Container::new(
                        TextInput::new("Name", &self.name_input)
                            .on_input(|input| StreamersTableMessage::InputName(input))
                        ).width(Length::FillPortion(2)).center_x()
                    )
                    .push(
                        Container::new(TextInput::new("IP Address", &self.ip_input)
                            .on_input(|input| StreamersTableMessage::InputIp(input))).width(Length::FillPortion(2)).center_x()
                    )
                    .push(
                        Container::new(
                            Row::new()
                            .push(
                                Button::new(Svg::from_path("../assets/checkmark.svg"))
                                    .width(Length::Fixed(30.0))
                                    .height(Length::Fixed(30.0))
                                    .on_press(StreamersTableMessage::CheckModifications((None, CudEnum::Create)))
                                )
                            .push(
                                Button::new(Svg::from_path("../assets/close.svg"))
                                    .width(Length::Fixed(30.0))
                                    .height(Length::Fixed(30.0))
                                    .on_press(StreamersTableMessage::ToggleAdd)
                            ).spacing(20)
                        ).width(Length::FillPortion(1)).center_x()
                    ).spacing(50)
                ).padding(50).into();
                col = col.push(new_record_input);
            },
            _ => {
                col = col.push(add_button);
            }
        }
        
        let main_content = match self.state{
            StreamersTableStateEnum::Error(message) => {
                InputErrorBanner::overlay(message,col, StreamersTableMessage::CloseBanner)
            },
            _ => {
                Container::new(col).center_y().center_x().into()
            }
        };
        let main_fit_content = Container::new(main_content).height(Length::FillPortion(5)).center_y();
        let  exit_button = 
            Container::new(
            Button::new(Text::new("Back").horizontal_alignment(Horizontal::Center))
                            .padding(10)
                            .width(Length::Fixed(150.0))
                            .on_press(StreamersTableMessage::Exit)
            ).height(Length::FillPortion(2)).center_y();
        Container::new(Column::new().push(main_fit_content).push(exit_button).width(Length::Fill).align_items(Alignment::Center)).into()
        
    }

    fn view_record(&self, record: (String, (String, String))) -> Element<StreamersTableMessage>{
        let row: Row<'_, StreamersTableMessage, _>  ;
        if self.editing_id.is_some() && &record.0 == self.editing_id.as_ref().unwrap(){
            row = Row::new()
                .push(
                    Container::new(
                        TextInput::new(&record.1.0, &self.name_input)
                            .on_input(|input| StreamersTableMessage::InputName(input))
                        ).width(Length::FillPortion(2)).center_x()
                )
                .push(
                    Container::new(TextInput::new(&record.1.1, &self.ip_input)
                    .on_input(|input| StreamersTableMessage::InputIp(input))).width(Length::FillPortion(2)).center_x()
                )
                .push(
                    Container::new(Button::new(Svg::from_path("../assets/checkmark.svg"))
                        .width(Length::Fixed(30.0))
                        .height(Length::Fixed(30.0))
                        .on_press(StreamersTableMessage::CheckModifications((Some(record.0), CudEnum::Update)))).width(Length::FillPortion(1)).center_x()
                );    
        }else{
            row = Row::new()
                .push(
                    Container::new(Text::new(record.1.0).size(20)).width(Length::FillPortion(2)).center_x()
                )
                .push(
                    Container::new(Text::new(record.1.1).size(20)).width(Length::FillPortion(2)).center_x()
                )
                .push(
                    Container::new(
                        Row::new()
                            .push(
                                Button::new(Svg::from_path("../assets/pencil.svg"))
                                    .width(Length::Fixed(30.0))
                                    .height(Length::Fixed(30.0))
                                    .on_press(StreamersTableMessage::Modify(record.0.clone()))
                            )
                            .push(
                                Button::new(Svg::from_path("../assets/trash.svg"))
                                    .width(Length::Fixed(30.0))
                                    .height(Length::Fixed(30.0))
                                    .on_press(StreamersTableMessage::CheckModifications((Some(record.0.clone()), CudEnum::Delete)))
                            ).spacing(20)
                ).width(Length::FillPortion(1)).center_x()
                );
        }

        Container::new(row.align_items(Alignment::Center).spacing(50)).style(theme::Container::Custom(RecordStyle.into())).into()
        
    }
    
    pub fn get_users(&self) -> HashMap<String, (String, String)>{
        let mut stmt = self.db_connection.prepare("SELECT id, name, ip FROM streamers").unwrap();
        let user_iter = stmt.query_map([], |row| {
                Ok((row.get::<_,String>(0)?, (row.get::<_, String>(1)?, row.get::<_, String>(2)?)))
            }).unwrap();
        
        let mut users = HashMap::new();
        for user in user_iter {
            let (id, user_data) = user.unwrap();
            users.insert(id, user_data);
        }
       users
    }

    fn check_modifications(&self, id: Option<String>, opt: CudEnum) -> Result<(Option<String>, String, String), InputError> {
        let streamers = self.get_users();
        let new_name;
        let new_ip;
        match opt{
            CudEnum::Create => {
                if &self.name_input == "" {
                    return Err(InputError::NotAName);
                }else {
                    new_name = self.name_input.clone().to_ascii_lowercase();
                }
                match &self.ip_input.parse::<Ipv4Addr>(){
                    Err(_) => {
                        return Err(InputError::NotAnIp);
                    },
                    Ok(_) => {new_ip = self.ip_input.clone();}
                }
                for record in streamers.iter(){
                        if new_name == record.1.0 {
                            return Err(InputError::NameAlreadyPresent);
                        }else if new_ip == record.1.1{
                            return Err(InputError::IpAlreadyPresent);
                        }
                }
            },
            CudEnum::Update => {
                let streamer;
                match streamers.get(id.as_ref().unwrap()){
                    Some(s) => streamer = s,
                    None => return Err(InputError::IdNotFound)
                };
                new_name = if &self.name_input == "" {streamer.0.clone()} else {self.name_input.clone().to_ascii_lowercase()};
                new_ip =  if &self.ip_input == "" {streamer.1.clone()} else {self.ip_input.clone()};
                for record in streamers.iter(){
                    if record.0 != id.as_ref().unwrap() {
                        if new_name == record.1.0 {
                            return Err(InputError::NameAlreadyPresent);
                        }else if new_ip == record.1.1{
                            return Err(InputError::IpAlreadyPresent);
                        }
                    }
                }
                match new_ip.parse::<Ipv4Addr>(){
                    Err(_) => {
                        return Err(InputError::NotAnIp);
                    }
                    _ => {}
                }
            },
            CudEnum::Delete => {
                match streamers.get(id.as_ref().unwrap()){
                    Some(_) => { new_name = "".to_string(); new_ip = "".to_string();},
                    None => return Err(InputError::IdNotFound)
                };
            },
        }

        Ok((id, new_name, new_ip))
    }

}



fn add_record(conn: &Connection, name: String, ip: String){
    let id = Uuid::new_v4();
    let _ = conn.execute(
            "INSERT INTO streamers (id, name, ip) VALUES (?1, ?2, ?3)",
            params![&id.to_string(), &name.to_ascii_lowercase(), &ip],
        );
}

fn delete_record(conn: &Connection, id: String) {
    match conn.execute(
        "DELETE FROM streamers WHERE id = ?1",
        params![id],
    ){
        Ok(_) => {},
        Err(e) => {println!("Error deleting record: {}", e);}
    }
}

fn modify_record(conn: &Connection, name: String, ip: String, id: String) {
    let _ = conn.execute(
        "UPDATE streamers SET name = ?1, ip = ?2 WHERE id = ?3",
        params![name, ip, id],
    );
}

#[derive(Debug, Clone, Default)]
struct RecordStyle;

impl iced::widget::container::StyleSheet for RecordStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Color::TRANSPARENT.into()),
            border_width: 1.0,
            text_color: Some(Color::WHITE),
            border_color: Color::from_rgb8(40, 41, 44),
            ..Default::default()
        }
    }
}
impl Into<Box<dyn iced::widget::container::StyleSheet<Style=Theme>>> for RecordStyle {
    fn into(self) -> Box<dyn iced::widget::container::StyleSheet<Style=Theme>> {
        Box::new(RecordStyle)
    }
}


