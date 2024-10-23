use std::collections::HashMap;
use rand::prelude::*;

use crate::game::resource::ResourceCard;
use crate::game::development::DevelopmentCard;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Bank {
    resource_cards: HashMap<ResourceCard, usize>,
    development_cards: [DevelopmentCard; 25],
    development_card_pointer: usize
}

#[allow(non_snake_case)]
impl Bank {
    // Initializes the resource cards and the development cards.
    pub fn new() -> Bank {
        let development_card_pointer = 0;

        // Initialize resource cards
        let mut resource_cards = HashMap::new();
        resource_cards.insert(ResourceCard::Ore, 19);
        resource_cards.insert(ResourceCard::Wheat, 19);
        resource_cards.insert(ResourceCard::Sheep, 19);
        resource_cards.insert(ResourceCard::Brick, 19);
        resource_cards.insert(ResourceCard::Lumber, 19);

        // Initialize development cards.
        let mut development_cards: [DevelopmentCard; 25] = core::array::from_fn(|index| {
            if index < 13{
                DevelopmentCard::Knight
            } else if index < 16 {
                DevelopmentCard::RoadBuilding
            } else if index < 18 {
                DevelopmentCard::YearOfPlenty
            } else if index < 20 {
                DevelopmentCard::Monopoly
            } else  {
                DevelopmentCard::VictoryPoint
            }
        });
        let mut rng = rand::thread_rng();
        development_cards.shuffle(&mut rng);

        Bank {
            development_card_pointer,
            resource_cards,
            development_cards
        }
    }

    // Returns the top of the development card stack.
    pub fn drawDevelopmentCard(&mut self) -> Option<&DevelopmentCard> {
        if self.development_card_pointer == 25 {
            return None
        }
        let development_card = &self.development_cards[self.development_card_pointer];
        self.development_card_pointer += 1;
        Some(development_card)
    }

    // Returns true if the number of resource cards is able to be drawn, else false.
    // If the resource cards can be drawn, they are removed from the deck.
    pub fn drawNumberOfResourceCards(&mut self, resource_card: ResourceCard, amount: usize) -> bool {
        match self.resource_cards.get(&resource_card) {
            Some(num_in_pile) => {
                if *num_in_pile >= amount {
                    self.resource_cards.insert(resource_card, num_in_pile-amount);
                    true
                } else {
                    false
                }
            },
            None => false
        }
    }

    // Replaces the resource cards in the bank.
    pub fn replaceResourceCard(&mut self, resource_card: ResourceCard, amount: usize) {
        match self.resource_cards.get(&resource_card) {
            Some(num_in_pile) => {self.resource_cards.insert(resource_card, num_in_pile + amount);},
            None => {}
        }
    }

    pub fn amountOfResource(&self, resource: ResourceCard) -> usize {
        return *self.resource_cards.get(&resource).unwrap();
    }
}