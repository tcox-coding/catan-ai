use serde::{Serialize, Deserialize};

// (Position, Player)
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum Building {
    Settlement(usize, usize),
    City(usize, usize),
    Road(usize, usize)
}