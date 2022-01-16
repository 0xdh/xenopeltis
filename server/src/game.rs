use rand::Rng;
use std::collections::VecDeque;
use xenopeltis_common::{Color, Direction, Field};

#[derive(Clone, Debug)]
struct Game {
    state: Vec<Vec<Option<Field>>>,
    snake: VecDeque<(isize, isize)>,
    direction: Direction,
}

impl Game {
    fn new() -> Self {
        let mut state = vec![vec!(None; 10); 10];
        state[5][5] = Some(Field::Snake(Color::default()));
        let mut snake = VecDeque::new();
        snake.push_back((5, 5));

        Game {
            state,
            direction: Direction::Right,
            snake,
        }
    }

    fn add_food(&mut self) {
        let mut rng = rand::thread_rng();

        loop {
            let y = rng.gen_range(0..self.state.len());
            let x = rng.gen_range(0..self.state[0].len());

            if self.state[y][x] == None {
                self.state[y][x] = Some(Field::Food);
                break;
            }
        }
    }

    fn tick(&mut self) -> bool {
        let head = self.snake.back().unwrap();
        let dir = self.direction.offset();
        let next = (dir.0 + head.0, dir.1 + head.1);
        let element = self
            .state
            .get(next.0 as usize)
            .map(|r| r.get(next.1 as usize));

        if element == None {
            return false;
        }

        let element = element.unwrap().unwrap();

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
