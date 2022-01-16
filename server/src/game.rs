use rand::Rng;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::net::SocketAddr;
use xenopeltis_common::{Color, Direction, Field};

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
    snake: VecDeque<(isize, isize)>,
    direction: Direction,
    players: BTreeMap<SocketAddr, Player>,
}

impl Game {
    pub fn new() -> Self {
        let mut state = vec![vec!(None; 10); 10];
        state[5][5] = Some(Field::Snake(Color::default()));
        let mut snake = VecDeque::new();
        snake.push_back((5, 5));

        Game {
            state,
            direction: Direction::Right,
            snake,
            players: BTreeMap::new(),
        }
    }

    pub fn add_player(&mut self, addr: SocketAddr) {
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

    fn tick(&mut self) -> bool {
        let head = self.snake.back().unwrap();
        let dir = self.direction.offset();
        let next = (dir.0 + head.0, dir.1 + head.1);
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
                self.snake.push_back(next);
                self.state[next.0 as usize][next.1 as usize] = Some(Field::Snake(Color::default()));
                self.add_food();
            }
            Some(Field::Snake(_)) => {
                return false;
            }
            None => {
                self.snake.push_back(next);
                self.state[next.0 as usize][next.1 as usize] = Some(Field::Snake(Color::default()));
                let last = self.snake.pop_front().unwrap();
                self.state[last.0 as usize][last.1 as usize] = None;
            }
            _ => unreachable!(),
        }

        true
    }
}
