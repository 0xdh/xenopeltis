mod game;

use anyhow::Result;
use futures::prelude::*;
use game::Game;
use log::*;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;
use tokio::io::AsyncWriteExt;
use tokio::net::{tcp::OwnedWriteHalf, TcpSocket, TcpStream};
use tokio::sync::broadcast::{channel, Receiver};
use tokio::sync::Mutex;
use tokio_serde::{formats::SymmetricalBincode, SymmetricallyFramed};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use xenopeltis_common::*;

#[derive(StructOpt)]
struct Options {
    #[structopt(long, short, default_value = "0.0.0.0:8000")]
    listen: SocketAddr,
    #[structopt(long, short, default_value = "20")]
    rows: usize,
    #[structopt(long, short, default_value = "80")]
    cols: usize,
    #[structopt(long, short, default_value = "2")]
    food: usize,
    #[structopt(long, short, default_value = "500")]
    tick: u64,
}

async fn handler_write(
    game: Arc<Mutex<Game>>,
    writer: OwnedWriteHalf,
    peer: SocketAddr,
    mut events: Receiver<ServerMessage>,
) {
    let framed_writer = FramedWrite::new(writer, LengthDelimitedCodec::new());
    let mut framed = SymmetricallyFramed::new(
        framed_writer,
        SymmetricalBincode::<ServerMessage>::default(),
    );

    let game_lock = game.lock().await;
    let messages = game_lock.messages_initial(peer);
    drop(game_lock);

    for message in messages {
        framed.send(message).await;
    }

    loop {
        match events.recv().await {
            Ok(event) => {
                framed.send(event).await;
            }
            _ => {}
        }
    }
}

async fn handler(game: Arc<Mutex<Game>>, mut connection: TcpStream, peer: SocketAddr) {
    info!("Connection from {}", peer);
    let mut game_lock = game.lock().await;
    let events = game_lock.player_add(peer);
    drop(game_lock);

    let (reader, writer) = connection.into_split();
    tokio::spawn(handler_write(game.clone(), writer, peer, events));

    let framed_reader = FramedRead::new(reader, LengthDelimitedCodec::new());
    let mut framed = SymmetricallyFramed::new(
        framed_reader,
        SymmetricalBincode::<ClientMessage>::default(),
    );

    loop {
        match framed.try_next().await {
            // we got a valid message, handle it
            Ok(Some(message)) => {
                info!("Message from {}: {:?}", peer, message);
                let mut game_lock = game.lock().await;
                game_lock.handle(peer, &message).await;
            }
            // end of stream (client closed connection)
            Ok(None) => break,
            // some kind of error happened, log it
            Err(e) => {
                error!("Error from {}: {}", peer, e);
                break;
            }
        }
    }

    let mut game_lock = game.lock().await;
    game_lock.player_remove(&peer);
}

async fn game_loop(game: Arc<Mutex<Game>>, duration: Duration) {
    let mut interval = tokio::time::interval(duration);
    loop {
        interval.tick().await;
        info!("Running game tick");
        let mut game_lock = game.lock().await;
        game_lock.tick();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let options = Options::from_args();

    let socket = TcpSocket::new_v4()?;
    socket.bind(options.listen)?;

    let listener = socket.listen(1024)?;

    let mut game = Game::new(options.rows, options.cols);

    // add food
    for x in 0..options.food {
        game.add_food();
    }

    let game = Arc::new(Mutex::new(game));

    tokio::spawn(game_loop(game.clone(), Duration::from_millis(options.tick)));

    loop {
        let (stream, peer) = listener.accept().await?;
        tokio::spawn(handler(game.clone(), stream, peer));
    }

    Ok(())
}
