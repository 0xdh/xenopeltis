use structopt::StructOpt;
use std::net::SocketAddr;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::*;
use std::io::{Write, stdout, stdin};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::net::{tcp::OwnedReadHalf, TcpStream};
use xenopeltis_common::*;
use anyhow::Result;
use tokio_serde::{formats::SymmetricalBincode, SymmetricallyFramed};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use futures::prelude::*;
use termion_input_tokio::TermReadAsync;

#[derive(StructOpt, Clone, Debug)]
pub struct Options {
    server: SocketAddr,
}

pub struct State {
    data: BTreeMap<Coordinate, Field>,
    data_dirty: BTreeMap<Coordinate, Field>,
    exit: bool,
}

impl State {
    pub fn new() -> Self {
        State {
            data: BTreeMap::new(),
            data_dirty: BTreeMap::new(),
            exit: false,
        }
    }
}

pub async fn handle_stream(state: Arc<Mutex<State>>, reader: OwnedReadHalf) -> Result<()> {
    let framed_reader = FramedRead::new(reader, LengthDelimitedCodec::new());
    let mut framed = SymmetricallyFramed::new(
        framed_reader,
        SymmetricalBincode::<ServerMessage>::default(),
    );

    loop {
        match framed.try_next().await {
            Ok(Some(message)) => {
            }
            _ => {}
        }
    }

    Ok(())
}

pub async fn draw_task(state: Arc<Mutex<State>>) {
    draw_task_run(state).await.unwrap();
}

pub async fn draw_task_run(state: Arc<Mutex<State>>) -> Result<()> {
    let mut screen = AlternateScreen::from(stdout().into_raw_mode()?);
    write!(screen, "{}", termion::cursor::Hide)?;
    screen.flush().unwrap();

    let mut interval = tokio::time::interval(Duration::from_millis(20));
    loop {
        interval.tick().await;
        let mut state_lock = state.lock().await;
        if state_lock.exit {
            break;
        }

        // draw dirty fields
        for (coordinate, field) in std::mem::take(&mut state_lock.data_dirty).iter() {
            state_lock.data.insert(*coordinate, *field);
        }
    }

    write!(screen, "{}", termion::cursor::Show)?;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let options = Options::from_args();

    let state = Arc::new(Mutex::new(State::new()));
    let stream = TcpStream::connect(options.server).await?;

    let (reader, writer) = stream.into_split();
    tokio::spawn(handle_stream(state.clone(), reader));

    let framed_writer = FramedWrite::new(writer, LengthDelimitedCodec::new());
    let mut framed = SymmetricallyFramed::new(
        framed_writer,
        SymmetricalBincode::<ClientMessage>::default(),
    );

    let draw_task = tokio::spawn(draw_task(state.clone()));

    let stdin = stdin();
    let mut keys = tokio::io::stdin().keys_stream();
    loop {
        let key = keys.try_next().await?.unwrap();
        match key {
            Key::Char('q') => {
            },
            Key::Left | Key::Right | Key::Up | Key::Down => {
                framed.send(ClientMessage::Direction(DirectionMessage {
                    direction: match key {
                        Key::Left => Direction::Left,
                        Key::Right => Direction::Right,
                        Key::Up => Direction::Up,
                        Key::Down => Direction::Down,
                        _ => unreachable!()
                    },
                })).await;
            },
            _ => {}
        }
    }

    let _ = draw_task.await;

    Ok(())
}
