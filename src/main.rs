extern crate ffmpeg_next as ffmpeg;

use std::time::{Instant, Duration};

use iced::{
    canvas::{
        self, Canvas, LineCap, Path, Stroke, Drawable,
        layer::{
            Cache,
        }
    },
    Application, Element, Text, Container,
    Length, Align, button, Row, Button,
    Image, executor, Command, Subscription,
    image, Color, Point, Vector, Size, Settings,
    Column,
};

mod preview;

enum Flag {
    Input(ffmpeg::format::context::Input)
}

struct MoviePlayer {
    play_button: button::State,
    pause_button: button::State,
    stop_button: button::State,
    video_provider: preview::VideoProvider,
    input: ffmpeg::format::context::Input,
}

impl<'a> Application for MoviePlayer {
    type Executor = executor::Default;
    type Message = VideoPlayerMessage;
    type Flags = Flag;

    fn new(flags: Flag) -> (Self, Command<Self::Message>) {
        match flags {
            Flag::Input(input) => (
                MoviePlayer {
                    play_button: Default::default(),
                    pause_button: Default::default(),
                    stop_button: Default::default(),
                    video_provider: preview::VideoProvider::new(input.streams().best(ffmpeg::media::Type::Video).unwrap()).unwrap(),
                    input,
                },
                Command::none()
            )
        }
    }

    fn title(&self) -> String {
        String::from("Movie Player")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            VideoPlayerMessage::Tick(_tick) => {
                if self.video_provider.next_frame().is_err() {
                    let video_stream_index = self.input.streams().best(ffmpeg::media::Type::Video).unwrap().index();
                    if let Some(packet) = self.input.packets().next() {
                        if packet.0.index() == video_stream_index {
                            self.video_provider.send_packet_to_decoder(&packet.1);
                        }
                    }
                    else {
                        self.video_provider.send_eof_to_decoder();
                    }
                }

            },
            _ => {},
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let content = Column::new()
        .spacing(20)
        .align_items(Align::Center)
        .push(
            self.video_provider.view()
        )
        .push(
            Row::new()
                .spacing(20)
                .align_items(Align::Center)
                .push(
                Button::new(&mut self.play_button, Text::new("Play"))
                )
                .push(
                Button::new(&mut self.pause_button, Text::new("Pause"))
                )
                .push(
                Button::new(&mut self.stop_button, Text::new("Stop"))
            )
        );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(Duration::from_micros(1666)).map(VideoPlayerMessage::Tick)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VideoPlayerMessage {
    Play,
    Pause,
    Stop,
    Tick(Instant),
}

fn main() {
    ffmpeg::init().unwrap();
    ffmpeg::log::set_level(ffmpeg::log::Level::Info);
    let path = String::from("Islands - 2119.mp4");
    let input = ffmpeg::format::input(&path).expect("Cannot open file.");
    MoviePlayer::run(Settings::with_flags(Flag::Input(input)));
}

mod time {
    use iced::futures;

    pub fn every(
        duration: std::time::Duration,
    ) -> iced::Subscription<std::time::Instant> {
        iced::Subscription::from_recipe(Every(duration))
    }

    struct Every(std::time::Duration);

    impl<H, I> iced_native::subscription::Recipe<H, I> for Every
    where
        H: std::hash::Hasher,
    {
        type Output = std::time::Instant;

        fn hash(&self, state: &mut H) {
            use std::hash::Hash;

            std::any::TypeId::of::<Self>().hash(state);
            self.0.hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: futures::stream::BoxStream<'static, I>,
        ) -> futures::stream::BoxStream<'static, Self::Output> {
            use futures::stream::StreamExt;

            async_std::stream::interval(self.0)
                .map(|_| std::time::Instant::now())
                .boxed()
        }
    }
}
