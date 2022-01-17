use serde::{Serialize, Deserialize};

/// Messages coming from the client to the server.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientMessage {
    Direction(DirectionMessage),
    Quit,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
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
    GameState(GameStateMessage),

}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GameStateMessage {
    Playing,
    Won,
    Lost,
}

/// RGB color.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

/// What is in a field?
///
/// Can be empty, apple (edible) or snake. Snakes are differentiated
/// by their color.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Empty,
    Wall,
    Food,
    Snake(Color),
}

/// Represents a coordinate on the game field
#[derive(Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Coordinate {
    pub row: usize,
    pub col: usize,
}

impl Coordinate {
    pub fn new(row: usize, col: usize) -> Coordinate {
        Coordinate {
            row,
            col,
        }
    }
}

/// A field has just changed the field type
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FieldChangeMessage {
    pub coordinate: Coordinate,
    pub field: Field,
}
