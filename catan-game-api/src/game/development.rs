use serde::{Serialize, Deserialize};

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum DevelopmentCard {
    Knight,
    RoadBuilding,
    YearOfPlenty,
    Monopoly,
    VictoryPoint
}

impl DevelopmentCard {
    pub fn value(&self) -> usize {
        match *self {
            DevelopmentCard::Knight => 0,
            DevelopmentCard::Monopoly => 1,
            DevelopmentCard::RoadBuilding => 2,
            DevelopmentCard::VictoryPoint => 3,
            DevelopmentCard::YearOfPlenty => 4
        }
    }
}