use core::error;

use thiserror::Error;
use iced::{Element, Length, Alignment, Color, Theme, theme, alignment::Vertical};
use iced::widget::{ Button, Column, Container, Row, Text};

#[derive(Debug, Error, Clone, Copy)]
pub enum InputError {

    #[error("The name is already present.")]
    NameAlreadyPresent,

    #[error("The IP address is already present.")]
    IpAlreadyPresent,

    #[error("The provided value is not a valid Id.")]
    IdNotFound,

    #[error("The provided value is not a valid name.")]
    NotAName,

    #[error("The provided value is not a valid IP address.")]
    NotAnIp,

    #[error("The provided ip value is not in the same LAN.")]
    NotInSameLan,

    #[error("The provided value has multiple matches.")]
    MultipleMatches,

    #[error("No value provided.")]
    NoValue,
}

pub trait Banner<'a> {
    type ExtMessage: Clone + 'a;

    fn overlay(
        message: InputError,
        content: Column<'a, Self::ExtMessage>,
        close_message: Self::ExtMessage,
    ) -> Element<'_, Self::ExtMessage> {
        let error_text;
        match message{
            InputError::NameAlreadyPresent => {
                error_text = "Name is already present";
            },
            InputError::IpAlreadyPresent => {
                error_text = "Ip is already present";
            },
            InputError::IdNotFound => {
                error_text = "Id not found";
            },
            InputError::NotAName => {
                error_text = "Not a name";
            },
            InputError::NotAnIp => {
                error_text = "Not an IP address";
            },
            InputError::NotInSameLan => {
                error_text = "Not in the same LAN";
            },
            InputError::MultipleMatches => {
                error_text = "Multiple matches";
            },
            InputError::NoValue => {
                error_text = "No value provided";
            },
        }
        let overlay = Container::new(
            Row::new()
                .spacing(10)
                .padding(10)
                .align_items(Alignment::Center)
                .push(Text::new(error_text).size(16))
                .push(
                    Button::new("Close")
                        .on_press(close_message)
                        .padding(5),
                ),
        )
        .width(Length::Fill)
        .padding(10)
        .style(theme::Container::Custom(BannerStyle.into()));

        // Stack the banner on top of the content
        let content = Column::new()
            .push(overlay)
            .push(Container::new(content).padding(20).height(Length::Fill).center_y()).align_items(Alignment::Center);

        Element::from(Container::new(content))
        
    }
}

#[derive(Debug, Clone, Default)]
struct BannerStyle;

impl iced::widget::container::StyleSheet for BannerStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Color::from_rgb(0.9, 0.1, 0.1).into()), // Red background
            border_radius: 5.0.into(),
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}
impl Into<Box<dyn iced::widget::container::StyleSheet<Style=Theme>>> for BannerStyle {
    fn into(self) -> Box<dyn iced::widget::container::StyleSheet<Style=Theme>> {
        Box::new(BannerStyle)
    }
}