use rand::Rng;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::net::SocketAddr;
use tokio::sync::broadcast::{channel, Sender};
use xenopeltis_common::{Color, Direction, Field, ServerMessage};

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

    pub fn player_add(&mut self, addr: SocketAddr) {
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
    }

    pub fn direction_set(&mut self, addr: &SocketAddr, dir: Direction) {
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

    fn state_set(&mut self, row: usize, col: usize, field: Field) {
        self.state[row][col] = Some(field);
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
                self.state[next.0 as usize][next.1 as usize] = Some(Field::Snake(Color::default()));
                self.add_food();
            }
            Some(Field::Snake(_)) => {
                return false;
            }
            None => {
                player.snake.push_back((next.0 as usize, next.1 as usize));
                self.state[next.0 as usize][next.1 as usize] = Some(Field::Snake(Color::default()));
                let last = player.snake.pop_front().unwrap();
                self.state[last.0 as usize][last.1 as usize] = None;
            }
            _ => unreachable!(),
        }

        true
    }
}
