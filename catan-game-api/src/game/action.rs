use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Action {
    pub action_type: ActionType,
    pub action_metadata: [usize; 10]
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum ActionType {
    RollDice,
    PlaceRobber,
    PlaySettlement,
    PlayRoad,
    PlayCity,
    OfferTrade,
    AcceptTrade,
    DeclineTrade,
    PlayDevelopmentCard,
    DrawDevelopmentCard,
    Discard,
    EndTurn
}

// impl ActionType {
//     fn value(&self) -> i32{
//         match *self {
//             ActionType::RollDice => 0,
//             ActionType::PlaceRobber => 1,
//             ActionType::PlaySettlement => 2,
//             ActionType::PlayRoad => 3,
//             ActionType::PlayCity => 4,
//             ActionType::OfferTrade => 5,
//             ActionType::AcceptTrade => 6,
//             ActionType::DeclineTrade => 7,
//             ActionType::PlayDevelopmentCard => 8,
//             ActionType::DrawDevelopmentCard => 9,
//             ActionType::Discard => 10,
//             ActionType::EndTurn => 11
//         }
//     }
// }