use iced::{Settings, Application, Command, Element, Theme};
use iced::widget::{button::Button, image, text::Text, container::Container, Column};
use crate::video_player::{VideoPlayer, Message};

struct MyApp {
    video_player: VideoPlayer,
}

impl Application for MyApp {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let video_player = VideoPlayer::new();
        (MyApp { video_player }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Video Player")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        // Delegate message handling to VideoPlayer
        self.video_player.update(message)
    }

    fn view(&self) -> Element<Self::Message> {
        let play_button = Button::new(Text::new("Play"))
            .on_press(Message::Play);

        let stop_button = Button::new(Text::new("Stop"))
            .on_press(Message::Stop);

        let video_frame = self.video_player.video_frame.lock().unwrap();
        let video_display = if let Some(handle) = &*video_frame {
            // Render video frame
            Container::new(image::Image::new(handle.clone()))
        } else {
            // Placeholder when no frame is available
            Container::new(Text::new("No video")).into()
        };

        Column::new()
            .push(play_button)
            .push(stop_button)
            .push(video_display)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        self.video_player.subscription()
    }
}

pub fn run_gui(){
    MyApp::run(Settings::default()).expect("Failed to start application");
}
