use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Port {
    ThreeToOne,
    Lumber,
    Ore,
    Wheat,
    Sheep,
    Brick,
}