use std::net::SocketAddr;
use structopt::StructOpt;

use anyhow::Result;
use futures::prelude::*;
use std::collections::BTreeMap;
use std::io::{stdin, stdout, Write};
use std::sync::Arc;
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::*;
use termion_input_tokio::TermReadAsync;
use tokio::net::{tcp::OwnedReadHalf, TcpStream};
use tokio::sync::Mutex;
use tokio_serde::{formats::SymmetricalBincode, SymmetricallyFramed};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use xenopeltis_common::*;

#[derive(StructOpt, Clone, Debug)]
pub struct Options {
    server: SocketAddr,
}

pub struct State {
    data: BTreeMap<Coordinate, Field>,
    data_dirty: BTreeMap<Coordinate, Field>,
    game_state: GameStateMessage,
    exit: bool,
}

impl State {
    pub fn new() -> Self {
        State {
            data: BTreeMap::new(),
            data_dirty: BTreeMap::new(),
            game_state: GameStateMessage::Playing,
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
            Ok(Some(ServerMessage::FieldChange(field_state))) => {
                let mut state_lock = state.lock().await;
                state_lock
                    .data_dirty
                    .insert(field_state.coordinate, field_state.field);
            }
            Ok(Some(ServerMessage::GameState(game_state))) => {
                let mut state_lock = state.lock().await;
                state_lock.game_state = game_state;
            }
            _ => {
                break;
            }
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
    let mut count: usize = 0;
    let mut dirty: usize = 0;
    let mut last_field: Field = Field::Empty;

    let mut interval = tokio::time::interval(Duration::from_millis(20));
    loop {
        interval.tick().await;
        let mut state_lock = state.lock().await;
        if state_lock.exit {
            break;
        }

        //write!(screen, "{}{}", termion::cursor::Goto(1, 1), count)?;
        //write!(screen, "{}{}: {:?}", termion::cursor::Goto(1, 2), dirty, last_field)?;

        // draw dirty fields
        for (coordinate, field) in std::mem::take(&mut state_lock.data_dirty).iter() {
            last_field = *field;
            let shape = match field {
                Field::Empty => " ",
                Field::Food => ".",
                Field::Snake(_) => "X",
                Field::Wall => "|",
            };
            write!(
                screen,
                "{}{}",
                termion::cursor::Goto(coordinate.col as u16 + 1, coordinate.row as u16 + 1),
                shape
            )?;
            state_lock.data.insert(*coordinate, *field);
            dirty += 1;
        }

        screen.flush()?;
        count += 1;
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
                let mut state_lock = state.lock().await;
                state_lock.exit = true;
                break;
            }
            Key::Left | Key::Right | Key::Up | Key::Down => {
                framed
                    .send(ClientMessage::Direction(DirectionMessage {
                        direction: match key {
                            Key::Left => Direction::Left,
                            Key::Right => Direction::Right,
                            Key::Up => Direction::Up,
                            Key::Down => Direction::Down,
                            _ => unreachable!(),
                        },
                    }))
                    .await;
            }
            _ => {}
        }
    }

    let _ = draw_task.await;

    Ok(())
}
