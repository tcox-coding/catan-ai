use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::game::resource::ResourceCard;

#[derive(Serialize, Deserialize)]
pub struct TradeOffer {
    pub player_offerer_id: usize,
    pub receiving_resources: HashMap<ResourceCard, usize>,
    pub giving_resources: HashMap<ResourceCard, usize>
}