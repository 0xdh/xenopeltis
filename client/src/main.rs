use std::net::SocketAddr;
use structopt::StructOpt;

use anyhow::Result;
use futures::prelude::*;
use std::collections::BTreeMap;
use std::io::{stdin, stdout, Write};
use std::sync::Arc;
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

type State = Arc<Mutex<BTreeMap<Coordinate, Field>>>;

pub async fn handle_stream(state: State, reader: OwnedReadHalf) -> Result<()> {
    let framed_reader = FramedRead::new(reader, LengthDelimitedCodec::new());
    let mut framed = SymmetricallyFramed::new(
        framed_reader,
        SymmetricalBincode::<ServerMessage>::default(),
    );

    loop {
        match framed.try_next().await {
            Ok(Some(message)) => {}
            _ => {}
        }
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let options = Options::from_args();

    let state: State = Arc::new(Mutex::new(BTreeMap::new()));
    let stream = TcpStream::connect(options.server).await?;

    let (reader, writer) = stream.into_split();
    tokio::spawn(handle_stream(state.clone(), reader));

    let framed_writer = FramedWrite::new(writer, LengthDelimitedCodec::new());
    let mut framed = SymmetricallyFramed::new(
        framed_writer,
        SymmetricalBincode::<ClientMessage>::default(),
    );

    let stdin = stdin();
    let mut screen = AlternateScreen::from(stdout().into_raw_mode()?);
    write!(screen, "{}", termion::cursor::Hide)?;
    screen.flush().unwrap();

    let mut keys = tokio::io::stdin().keys_stream();
    loop {
        let key = keys.try_next().await?.unwrap();
        match key {
            Key::Char('q') => break,
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
        screen.flush().unwrap();
    }
    write!(screen, "{}", termion::cursor::Show)?;

    Ok(())
}
