mod game;

use anyhow::Result;
use game::Game;
use std::net::SocketAddr;
use structopt::StructOpt;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpSocket, TcpStream};
use xenopeltis_common::*;

#[derive(StructOpt)]
struct Options {
    #[structopt(long, short, default_value = "0.0.0.0:8000")]
    listen: SocketAddr,
}

async fn handler(mut connection: TcpStream, peer: SocketAddr) {
    connection.write_all(b"Hello\n").await;
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::from_args();

    let socket = TcpSocket::new_v4()?;
    socket.bind(options.listen)?;

    let listener = socket.listen(1024)?;

    let game = Game::new();

    loop {
        let (stream, peer) = listener.accept().await?;
        tokio::spawn(handler(stream, peer));
    }

    Ok(())
}
