use anyhow::Result;
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
    state: Vec<Vec<Option<Field>>>,
    players: BTreeMap<SocketAddr, Player>,
    events: Sender<ServerMessage>,
}

impl Game {
    pub fn new() -> Self {
        let mut state = vec![vec!(None; 10); 10];
        state[5][5] = Some(Field::Snake(Color::default()));
        let (events, _) = channel(CHANNEL_SIZE);

        Game {
            state,
            players: BTreeMap::new(),
            events,
        }
    }

    pub fn player_add(&mut self, addr: SocketAddr) -> Receiver<ServerMessage> {
        let (x, y) = self.empty_field();
        let mut snake = VecDeque::new();
        snake.push_back((y, x));

        self.players.insert(
            addr,
            Player {
                snake,
                color: Color::default(),
                direction: Direction::default(),
            },
        );

        self.events.subscribe()
    }

    pub fn player_remove(&mut self, addr: &SocketAddr) {
        if let Some(player) = self.players.remove(addr) {
            for (row, col) in &player.snake {
                // TODO: remove snake fields
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

            if self.state[y][x] == None {
                return (y, x);
            }
        }
    }

    fn add_food(&mut self) {
        let (y, x) = self.empty_field();
        self.state[y][x] = Some(Field::Food);
    }

    fn state_set(&mut self, row: usize, col: usize, field: Field) -> Result<()> {
        self.state[row][col] = Some(field);
        self.events
            .send(ServerMessage::FieldChange(FieldChangeMessage {
                coordinate: Coordinate::new(row, col),
                field,
            }))?;
        Ok(())
    }

    pub fn tick(&mut self) {
        let players: Vec<_> = self.players.keys().cloned().collect();
        for player in players {
            if !self.player_tick(player) {
                self.players.remove(&player);
            }
        }
    }

    pub fn player_tick(&mut self, player: SocketAddr) -> bool {
        let player = self.players.get_mut(&player).unwrap();
        let head = player.snake.back().unwrap();
        let dir = player.direction.offset();
        let next = (dir.0 + head.0 as isize, dir.1 + head.1 as isize);
        let element = self
            .state
            .get(next.0 as usize)
            .and_then(|r| r.get(next.1 as usize));

        let element = match element {
            Some(value) => value,
            _ => return false,
        };

        match element {
            Some(Field::Wall) => {
                return false;
            }
            Some(Field::Food) => {
                player.snake.push_back((next.0 as usize, next.1 as usize));
                self.state_set(
                    next.0 as usize,
                    next.1 as usize,
                    Field::Snake(Color::default()),
                );
                self.add_food();
            }
            Some(Field::Snake(_)) => {
                return false;
            }
            None => {
                player.snake.push_back((next.0 as usize, next.1 as usize));
                let last = player.snake.pop_front().unwrap();

                self.state_set(
                    next.0 as usize,
                    next.1 as usize,
                    Field::Snake(Color::default()),
                );
                self.state_set(next.0 as usize, next.1 as usize, Field::Empty);
            }
            _ => unreachable!(),
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
