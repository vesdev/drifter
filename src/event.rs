use std::env;

use bevy::log::{error, info};
use bevy::render::color::Color;
use dotenvy::dotenv;
use eye::colorconvert::Device;
use eye::hal::format::PixelFormat;
use eye::hal::traits::{Context, Device as _, Stream};
use eye::hal::PlatformContext;
use tokio::sync::mpsc::{Receiver, Sender};
// use v4l::{io::traits::CaptureStream, video::Capture};

pub fn create_receiver() -> anyhow::Result<EventReceiver> {
    let (tx, rx) = tokio::sync::mpsc::channel(1);

    std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                {
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        capture_video(tx).await.unwrap();
                    });
                }
                poll_chat(tx).await.unwrap();
            })
    });

    Ok(EventReceiver { receiver: rx })
}

pub struct EventReceiver {
    receiver: Receiver<Event>,
}

impl EventReceiver {
    pub fn try_recv(&mut self) -> Option<Event> {
        self.receiver.try_recv().ok()
    }
}

pub enum Event {
    Ball { color: Color },
    CameraFeed { frame: Vec<u8> },
}

async fn capture_video(sender: Sender<Event>) -> anyhow::Result<()> {
    let ctx = PlatformContext::default();

    let devices = ctx.devices()?;

    let dev = ctx.open_device(&devices[0].uri).unwrap();
    let dev = Device::new(dev)?;
    let stream_descr = dev
        .streams()?
        .into_iter()
        .reduce(|s1, s2| {
            // Choose RGB with 8 bit depth
            if s1.pixfmt == PixelFormat::Rgb(24) && s2.pixfmt != PixelFormat::Rgb(24) {
                return s1;
            }

            // Strive for HD (1280 x 720)
            let distance = |width: u32, height: u32| {
                f32::sqrt(((1280 - width as i32).pow(2) + (720 - height as i32).pow(2)) as f32)
            };

            if distance(s1.width, s1.height) < distance(s2.width, s2.height) {
                s1
            } else {
                s2
            }
        })
        .unwrap();

    if stream_descr.pixfmt != PixelFormat::Rgb(24) {
        error!("No RGB3 streams available");
    }

    info!("Selected stream:\n{:?}", stream_descr);

    let mut stream = dev.start_stream(&stream_descr)?;

    loop {
        let frame = stream.next().unwrap().unwrap();
        let mut result =
            Vec::with_capacity(stream_descr.width as usize * stream_descr.height as usize * 4);

        for y in (0..stream_descr.height).rev() {
            for x in 0..stream_descr.width {
                let px = (x + y * stream_descr.width) as usize * 3;
                result.push(frame[px]);
                result.push(frame[px + 1]);
                result.push(frame[px + 2]);
                result.push(255);
            }
        }

        sender
            .send(Event::CameraFeed { frame: result })
            .await
            .unwrap();
    }
}

async fn poll_chat(sender: Sender<Event>) -> anyhow::Result<()> {
    dotenv()?;
    let channels = &[tmi::Channel::parse("#vesdeg".into())?];
    let mut client = tmi::Client::builder()
        .credentials(tmi::Credentials {
            nick: "vuekobot".into(),
            pass: env::var("TMI_PASS")?,
        })
        .connect()
        .await?;
    client.join_all(channels).await?;

    loop {
        let msg = client.recv().await?;
        match msg.as_typed()? {
            tmi::Message::Reconnect => {
                client.reconnect().await?;
                client.join_all(channels).await?;
            }
            tmi::Message::Ping(ping) => {
                client.pong(&ping).await?;
            }
            tmi::Message::Privmsg(msg) => {
                if let Some(msg) = msg.text().strip_prefix('!') {
                    let mut args = msg.split_whitespace();
                    #[allow(clippy::single_match)]
                    match args.next() {
                        Some("mpv") => {
                            let Some(url) = args.next() else {
                                continue;
                            };

                            std::process::Command::new("mpv").arg(url).output()?;
                        }
                        Some("ball") => {}
                        _ => {}
                    };
                    continue;
                };

                if msg.custom_reward_id() == Some("be22f712-8fd9-426a-90df-c13eae6cc6dc") {
                    let color = Color::hex(msg.text()).unwrap_or(Color::WHITE);

                    sender.send(Event::Ball { color }).await?;
                }
            }
            _ => {}
        };
    }
}
