use anyhow::Result;
use futures::{SinkExt, StreamExt};
use log::*;
use serde_json::{from_str, to_string};
use std::net::SocketAddr;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio_serde::{formats::Bincode, Framed};
use tokio_tungstenite::tungstenite::Message;
use tokio_util::codec::{Framed as FramedCodec, LengthDelimitedCodec};
use xenopeltis_common::*;

#[derive(StructOpt, Clone, Debug)]
struct Options {
    #[structopt(
        long,
        short,
        env = "XENOPELTIS_WEBSOCKET_LISTEN",
        default_value = "0.0.0.0:9000"
    )]
    listen: SocketAddr,
    #[structopt(
        long,
        short,
        env = "XENOPELTIS_WEBSOCKET_TARGET",
        default_value = "127.0.0.1:8000"
    )]
    target: SocketAddr,
}

async fn handle_connection(options: Arc<Options>, stream: TcpStream, peer: SocketAddr) {
    match handle_connection_real(options, stream, peer).await {
        Ok(()) => (),
        Err(e) => error!("Error in websocket connection: {}", e),
    }
}

async fn handle_connection_real(
    options: Arc<Options>,
    stream: TcpStream,
    peer: SocketAddr,
) -> Result<()> {
    let mut websocket = tokio_tungstenite::accept_async(stream).await?;
    let server_connection = TcpStream::connect(options.target).await?;
    let framed_connection = FramedCodec::new(server_connection, LengthDelimitedCodec::new());
    let mut messages: Framed<_, ServerMessage, ClientMessage, _> = Framed::new(
        framed_connection,
        Bincode::<ServerMessage, ClientMessage>::default(),
    );
    info!("Connected to server for {}", peer);

    loop {
        select! {
            message = messages.next() => {
                match message {
                    Some(Ok(message)) => {
                        debug!("Got message for peer {}: {:?}", peer, &message);
                        websocket.send(Message::Text(to_string(&message)?)).await?
                    }
                    _ => break,
                }
            }
            message = websocket.next() => {
                match message {
                    Some(Ok(Message::Text(message))) => {
                        debug!("Got websocket message for peer {}: {}", peer, &message);
                        let message: ClientMessage = from_str(&message)?;
                        messages.send(message).await?;
                    },
                    _ => break,
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let options = Arc::new(Options::from_args());
    debug!("Parsed options: {:?}", options);

    let listener = TcpListener::bind(options.listen).await?;

    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                info!("Accepted connection from {}", peer);
                tokio::spawn(handle_connection(options.clone(), stream, peer));
            }
            Err(e) => error!("Error accepting connection: {}", e),
        }
    }
}
