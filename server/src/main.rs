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
use tokio::sync::Mutex;
use tokio_serde::{formats::SymmetricalBincode, SymmetricallyFramed};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use xenopeltis_common::*;

#[derive(StructOpt)]
struct Options {
    #[structopt(long, short, default_value = "0.0.0.0:8000")]
    listen: SocketAddr,
}

async fn handler_write(writer: OwnedWriteHalf, peer: SocketAddr) {
    let framed_reader = FramedWrite::new(writer, LengthDelimitedCodec::new());
    let mut framed = SymmetricallyFramed::new(
        framed_reader,
        SymmetricalBincode::<ServerMessage>::default(),
    );

    framed
        .send(ServerMessage::GameState(GameStateMessage::Playing))
        .await
        .unwrap();
}

async fn handler(game: Arc<Mutex<Game>>, mut connection: TcpStream, peer: SocketAddr) {
    info!("Connection from {}", peer);
    let mut game_lock = game.lock().await;
    game_lock.player_add(peer);
    drop(game_lock);

    let (reader, writer) = connection.into_split();

    tokio::spawn(handler_write(writer, peer));

    let framed_reader = FramedRead::new(reader, LengthDelimitedCodec::new());
    let mut framed = SymmetricallyFramed::new(
        framed_reader,
        SymmetricalBincode::<ClientMessage>::default(),
    );

    loop {
        match framed.try_next().await {
            Ok(Some(message)) => {
                info!("Message from {}: {:?}", peer, message);
            }
            Ok(None) => break,
            Err(e) => {
                error!("Error from {}: {}", peer, e);
                break;
            }
        }
    }
}

async fn game_loop(game: Arc<Mutex<Game>>) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
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

    let game = Arc::new(Mutex::new(Game::new()));
    tokio::spawn(game_loop(game.clone()));

    loop {
        let (stream, peer) = listener.accept().await?;
        tokio::spawn(handler(game.clone(), stream, peer));
    }

    Ok(())
}
