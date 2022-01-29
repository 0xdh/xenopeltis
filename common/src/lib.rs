use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Messages coming from the client to the server.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientMessage {
    Direction(DirectionMessage),
    Restart,
    Quit,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Right
    }
}

impl Direction {
    pub fn offset(&self) -> (isize, isize) {
        match self {
            Direction::Up => (-1, 0),
            Direction::Down => (1, 0),
            Direction::Left => (0, -1),
            Direction::Right => (0, 1),
        }
    }

    pub fn opposite(&self) -> Direction {
        use Direction::*;
        match self {
            Up => Down,
            Down => Up,
            Left => Right,
            Right => Left,
        }
    }
}

/// Client pressed a direction key, lets server know
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DirectionMessage {
    pub direction: Direction,
}

/// Messages coming from the server to the client.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerMessage {
    FieldChange(FieldChangeMessage),
    PlayerState(PlayerStateMessage),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerStateMessage {
    pub state: PlayerState,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PlayerState {
    Playing,
    Won,
    Lost,
}

/// RGB color.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Blue,
    Cyan,
    Green,
    Magenta,
    Red,
    Yellow,
}

impl Distribution<Color> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Color {
        use Color::*;
        match rng.gen_range(0..6) {
            0 => Blue,
            1 => Cyan,
            2 => Green,
            3 => Magenta,
            4 => Red,
            5 => Yellow,
            _ => unreachable!(),
        }
    }
}

/// What is in a field?
///
/// Can be empty, apple (edible) or snake. Snakes are differentiated
/// by their color.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Empty,
    Wall,
    Food(bool),
    Snake(Color),
}

impl Field {
    pub fn food(&self) -> bool {
        match self {
            Field::Food(_) => true,
            _ => false,
        }
    }
}

/// Represents a coordinate on the game field
#[derive(Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Coordinate {
    pub row: usize,
    pub col: usize,
}

impl Coordinate {
    pub fn new(row: usize, col: usize) -> Coordinate {
        Coordinate { row, col }
    }
}

/// A field has just changed the field type
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FieldChangeMessage {
    pub coordinate: Coordinate,
    pub field: Field,
}
