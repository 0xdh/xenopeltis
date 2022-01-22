use anyhow::Result;
use log::*;
use rand::Rng;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::net::SocketAddr;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use xenopeltis_common::*;

const CHANNEL_SIZE: usize = 1024;

pub struct State<T: Clone> {
    data: Vec<Vec<T>>,
}

impl<T: Clone> State<T> {
    pub fn new(initial: T, rows: usize, cols: usize) -> Self {
        State {
            data: vec![vec![initial; cols]; rows],
        }
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&T> {
        self.data.get(row).and_then(|row| row.get(col))
    }

    pub fn set(&mut self, row: usize, col: usize, value: T) {
        self.data
            .get_mut(row)
            .and_then(|row| row.get_mut(col))
            .map(|v| *v = value)
            .unwrap();
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    snake: VecDeque<(usize, usize)>,
    color: Color,
    direction: Direction,
}

#[derive(Clone, Debug)]
pub struct Game {
    state: Vec<Vec<Field>>,
    players: BTreeMap<SocketAddr, Player>,
    events: Sender<ServerMessage>,
}

impl Game {
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut state = vec![vec!(Field::Empty; cols); rows];

        // draw walls
        for x in 0..rows {
            state[x][0] = Field::Wall;
            state[x][cols - 1] = Field::Wall;
        }

        for x in 0..cols {
            state[0][x] = Field::Wall;
            state[rows - 1][x] = Field::Wall;
        }

        let (events, _) = channel(CHANNEL_SIZE);

        Game {
            state,
            players: BTreeMap::new(),
            events,
        }
    }

    pub fn player_add(&mut self, peer: SocketAddr) -> Receiver<ServerMessage> {
        let color = Color::default();
        let (row, col) = self.empty_field();
        let mut snake = VecDeque::new();
        snake.push_back((row, col));
        self.state_set(row, col, Field::Snake(color));
        info!(
            "Adding player {} to ({}, {}) with color {:?}",
            peer, row, col, color
        );

        self.players.insert(
            peer,
            Player {
                snake,
                color,
                direction: Direction::default(),
            },
        );

        self.events.subscribe()
    }

    pub fn player_remove(&mut self, addr: &SocketAddr) {
        if let Some(player) = self.players.remove(addr) {
            for (row, col) in &player.snake {
                self.state_set(*row, *col, Field::Food);
            }
        }
    }

    pub fn player_direction(&mut self, addr: &SocketAddr, dir: Direction) {
        match self.players.get_mut(addr) {
            Some(player) => player.direction = dir,
            None => {}
        }
    }

    pub fn empty_field(&self) -> (usize, usize) {
        let mut rng = rand::thread_rng();

        loop {
            let y = rng.gen_range(0..self.state.len());
            let x = rng.gen_range(0..self.state[0].len());

            if self.state[y][x] == Field::Empty {
                return (y, x);
            }
        }
    }

    pub fn add_food(&mut self) {
        let (row, col) = self.empty_field();
        self.state_set(row, col, Field::Food);
    }

    fn state_set(&mut self, row: usize, col: usize, field: Field) -> Result<()> {
        self.state[row][col] = field;
        self.events
            .send(ServerMessage::FieldChange(FieldChangeMessage {
                coordinate: Coordinate::new(row, col),
                field,
            }))?;
        Ok(())
    }

    pub fn messages_initial(&self, peer: SocketAddr) -> Vec<ServerMessage> {
        let mut messages = vec![];

        messages.push(ServerMessage::GameState(GameStateMessage::Playing));

        for (row, cols) in self.state.iter().enumerate() {
            for (col, field) in cols.iter().enumerate() {
                if *field != Field::Empty {
                    messages.push(ServerMessage::FieldChange(FieldChangeMessage {
                        coordinate: Coordinate::new(row, col),
                        field: *field,
                    }));
                }
            }
        }

        messages
    }

    pub fn tick(&mut self) {
        let players: Vec<_> = self.players.keys().cloned().collect();
        for player in players {
            if !self.player_tick(player) {
                info!("Player {} removed", player);
                self.player_remove(&player);
            }
        }
    }

    pub fn player_tick(&mut self, peer: SocketAddr) -> bool {
        let player = self.players.get_mut(&peer).unwrap();
        let head = player.snake.back().unwrap();
        let dir = player.direction.offset();
        let next = (dir.0 + head.0 as isize, dir.1 + head.1 as isize);
        let element = self
            .state
            .get(next.0 as usize)
            .and_then(|r| r.get(next.1 as usize));

        let element = match element {
            Some(value) => value,
            None => {
                info!("Player {} left playing field", peer);
                return false;
            }
        };

        match element {
            Field::Wall => {
                info!("Player {} collided with wall", peer);
                return false;
            }
            Field::Food => {
                info!("Player {} got food", peer);
                player.snake.push_back((next.0 as usize, next.1 as usize));
                let color = player.color;
                self.state_set(next.0 as usize, next.1 as usize, Field::Snake(color));
                self.add_food();
            }
            Field::Snake(_) => {
                info!("Player {} hit snake", peer);
                return false;
            }
            Field::Empty => {
                info!("Player {} moves to ({}, {})", peer, next.0, next.1);
                player.snake.push_back((next.0 as usize, next.1 as usize));
                let last = player.snake.pop_front().unwrap();
                let color = player.color;

                self.state_set(next.0 as usize, next.1 as usize, Field::Snake(color));
                self.state_set(last.0 as usize, last.1 as usize, Field::Empty);
            }
        }

        true
    }

    pub async fn handle(&mut self, peer: SocketAddr, message: &ClientMessage) {
        use ClientMessage::*;
        match message {
            Direction(dir) => self.player_direction(&peer, dir.direction),
            _ => {}
        }
    }
}
