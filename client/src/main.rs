use std::net::SocketAddr;
use structopt::StructOpt;

use anyhow::Result;
use std::collections::BTreeMap;
use std::io::{stdin, stdout, Write};
use std::sync::Arc;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::*;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use xenopeltis_common::*;

#[derive(StructOpt, Clone, Debug)]
pub struct Options {
    server: SocketAddr,
}

type State = Arc<Mutex<BTreeMap<Coordinate, Field>>>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let options = Options::from_args();

    let state: State = Arc::new(Mutex::new(BTreeMap::new()));
    let stream = TcpStream::connect(options.server).await?;

    let stdin = stdin();
    let mut screen = AlternateScreen::from(stdout().into_raw_mode()?);
    write!(screen, "{}", termion::cursor::Hide)?;
    screen.flush().unwrap();

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') => break,
            _ => {}
        }
        screen.flush().unwrap();
    }
    write!(screen, "{}", termion::cursor::Show)?;

    Ok(())
}
