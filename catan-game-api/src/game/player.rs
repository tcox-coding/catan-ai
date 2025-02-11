use std::collections::{HashMap, VecDeque};
use rand::prelude::SliceRandom;
use serde::{Serialize, Deserialize};

use crate::game::action::Action;
use crate::game::development::DevelopmentCard;
use crate::game::resource::ResourceCard;

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: usize,
    pub num_unplaced_cities: usize,
    pub num_unplaced_settlements: usize,
    pub num_unplaced_roads: usize,
    pub resource_cards: HashMap<ResourceCard, usize>,
    development_cards: HashMap<DevelopmentCard, usize>,
    development_cards_drawn_this_turn: HashMap<DevelopmentCard, usize>,
    pub longest_road: bool,
    pub largest_army: bool,
    pub settlement_placements: Vec<usize>,
    pub road_placements: Vec<usize>,
    pub city_placements: Vec<usize>,
    action_queue: VecDeque<Action>,
    pub victory_points: usize,
    pub num_knights_played: usize,
    pub longest_road_length: usize,
}

#[allow(non_snake_case)]
impl Player {
    pub fn new(id: usize) -> Self {
        let mut resource_cards = HashMap::new();
        resource_cards.insert(ResourceCard::Brick, 0);
        resource_cards.insert(ResourceCard::Lumber, 0);
        resource_cards.insert(ResourceCard::Ore, 0);
        resource_cards.insert(ResourceCard::Sheep, 0);
        resource_cards.insert(ResourceCard::Wheat, 0);

        let mut development_cards = HashMap::new();
        development_cards.insert(DevelopmentCard::Knight, 0);
        development_cards.insert(DevelopmentCard::Monopoly, 0);
        development_cards.insert(DevelopmentCard::RoadBuilding, 0);
        development_cards.insert(DevelopmentCard::VictoryPoint, 0);
        development_cards.insert(DevelopmentCard::YearOfPlenty, 0);

        Player {
            id: id,
            num_unplaced_cities: 4,
            num_unplaced_settlements: 5,
            num_unplaced_roads: 15,
            resource_cards: resource_cards,
            development_cards: development_cards.clone(),
            development_cards_drawn_this_turn: development_cards.clone(),
            longest_road: false,
            largest_army: false,
            settlement_placements: vec![],
            road_placements: vec![],
            city_placements: vec![],
            action_queue: VecDeque::new(),
            victory_points: 0,
            num_knights_played: 0,
            longest_road_length: 0,
        }
    }

    // Removes the cards from the player's hand, returns true or false if successful or not.
    pub fn removeCardsFromHand(&mut self, resources: HashMap<ResourceCard, usize>) -> bool {
        // Checks if the cards can be removed from the hand.
        for (resource, amount) in resources.iter() {
            if !(*self.resource_cards.get(resource).unwrap() >= *amount) {
                return false;
            }
        }
        // Removes cards from hand.
        for (resource, amount) in resources.iter() {
            self.resource_cards.insert(
                resource.clone(),
                self.resource_cards.get(resource).unwrap() - *amount
            );
        }
        true
    }

    pub fn hasDevelopmentCard(&mut self, development_card: DevelopmentCard) -> bool {
        return self.development_cards.get(&development_card).is_some() &&
            *self.development_cards.get(&development_card).unwrap() > 0;
    }

    pub fn hasResourceCards(&mut self, resources: HashMap<ResourceCard, usize>) -> bool {
        // Checks if the cards can be removed from the hand.
        for (resource, amount) in resources.iter() {
            if !(*self.resource_cards.get(resource).unwrap() >= *amount) {
                return false;
            }
        }
        true
    }

    pub fn addDevelopmentCard(&mut self, development_card: DevelopmentCard) {
        self.development_cards_drawn_this_turn.insert(
            development_card.clone(),
            self.development_cards.get(&development_card.clone()).unwrap() + 1
        );
    }

    pub fn removeDevelopmentCard(&mut self, development_card: DevelopmentCard) {
        self.development_cards.insert(
            development_card.clone(),
            self.development_cards.get(&development_card.clone()).unwrap() - 1
        );
    }

    pub fn addResourceCards(&mut self, resources: HashMap<ResourceCard, usize>) {
        for (key, value) in resources {
            self.resource_cards.insert(key.clone(), self.resource_cards.get(&key).unwrap() + value);
        }
    }

    pub fn stealCard(&mut self) -> Option<ResourceCard> {
        let mut available_cards = vec![];
        for (resource, amount) in self.resource_cards.clone().into_iter() {
            for _ in 0..amount {
                available_cards.push(resource.clone());
            }
        }

        if available_cards.len() < 1 {
            return None;
        }

        let chosen_card = available_cards.choose(&mut rand::thread_rng()).unwrap().clone();
        self.resource_cards.insert(chosen_card.clone(), self.resource_cards.get(&chosen_card).unwrap() - 1);

        Some(chosen_card)
    }

    pub fn removeAllResourcesFromHand(&mut self, resource: ResourceCard) -> usize {
        let amount_in_hand = *self.resource_cards.get(&resource).unwrap() as usize;
        self.resource_cards.insert(resource, 0);
        amount_in_hand
    }

    pub fn addResourceCard(&mut self, resource: ResourceCard) {
        self.resource_cards.insert(resource, self.resource_cards.get(&resource).unwrap() + 1);
    }

    pub fn addResourceCardAmount(&mut self, resource: ResourceCard, amount: usize) {
        self.resource_cards.insert(resource, self.resource_cards.get(&resource).unwrap() + amount);
    }

    pub fn moveDevelopmentCards(&mut self) {
        for (development_card, amount) in &self.development_cards_drawn_this_turn {
            self.development_cards.insert(*development_card, *self.development_cards.get(&development_card).unwrap() + amount);
        }
    }
}