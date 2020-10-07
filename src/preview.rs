extern crate ffmpeg_next as ffmpeg;

use std::time::{Instant, Duration};

use ffmpeg::{
    codec, decoder, encoder, frame, log, media, picture, Dictionary, Packet, Rational,
    format::{
        self, Pixel,
    },
    software::scaling::{
        context::Context,
        flag::Flags,
    },
};

use iced::{
    button, futures, image, Align, Application, Button, Column, Command,
    Container, Element, Image, Length, Row, Settings, Text,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VideoError {
    CouldNotReadNextFrame,
}

pub struct VideoProvider {
    decoder: decoder::Video,
    frame_count: usize,
    frame_rate: Rational,
    last_frame_time: Instant,
    current_frame: image::Handle,
    packet_index: usize,
    context: Context,
}

impl VideoProvider {
    pub fn new(input_stream: format::stream::Stream) -> Result<VideoProvider, ffmpeg::Error> {
        let decoder = input_stream.codec().decoder().video()?;
        let format = decoder.format();
        let width = decoder.width();
        let height = decoder.height();
        let frame_rate = decoder.frame_rate().unwrap();
        Ok(VideoProvider {
            decoder,
            frame_rate,
            last_frame_time: Instant::now(),
            frame_count: 0,
            current_frame: image::Handle::from_memory(vec![0u8, 0u8, 0u8]),
            packet_index: 0,
            context: Context::get(
                format,
                width,
                height,
                Pixel::BGRA,
                width,
                height,
                Flags::BILINEAR,
            )?
        })
    }

    pub fn next_frame(&mut self) -> Result<(), VideoError> {
        let mut frame = frame::Video::empty();
        if self.decoder.receive_frame(&mut frame).is_ok() {
            self.frame_count += 1;
            let timestamp = frame.timestamp();
            let mut rgb_frame = frame::Video::empty();
            self.context.run(&frame, &mut rgb_frame).expect("Could not run context");
            rgb_frame.set_pts(timestamp);
            self.current_frame = image::Handle::from_pixels(rgb_frame.width(), rgb_frame.height(), rgb_frame.data(0).to_vec());
            Ok(())
        }
        else {
            Err(VideoError::CouldNotReadNextFrame)
        }
    }

    pub fn send_packet_to_decoder(&mut self, packet: &Packet) {
        self.decoder.send_packet(packet).unwrap();
    }

    pub fn send_eof_to_decoder(&mut self) {
        self.decoder.send_eof().unwrap();
    }

    pub fn view(&self) -> Element<crate::VideoPlayerMessage> {
        Row::new()
            .spacing(20)
            .align_items(Align::Center)
            .push(
                Image::new(self.current_frame.clone())
            )
            .into()
    }
}

