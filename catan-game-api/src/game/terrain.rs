
use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, PartialEq)]
pub enum Terrain {
    Plains,
    Forest,
    Mountains,
    Hills,
    Fields,
    Desert,
}