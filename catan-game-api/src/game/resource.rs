use serde::{Serialize, Deserialize};

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum ResourceCard {
    Ore,
    Wheat,
    Sheep,
    Brick,
    Lumber,
}

// impl ResourceCard {
//     pub fn value(&self) -> usize {
//         match *self {
//             ResourceCard::Ore => 0,
//             ResourceCard::Wheat => 1,
//             ResourceCard::Sheep => 2,
//             ResourceCard::Brick => 3,
//             ResourceCard::Lumber => 4
//         }
//     }
// }