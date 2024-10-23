use serde::{Serialize, Deserialize};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use rand::Rng;

use crate::game::player::Player;
use crate::game::board::Board;
use crate::game::bank::Bank;
use crate::game::action::Action;
use crate::game::building::Building;
use crate::game::action::ActionType;
use crate::game::trade_offer::TradeOffer;
use crate::game::resource::ResourceCard;
use crate::game::port::Port;

use super::development::DevelopmentCard;
use super::terrain::Terrain;

#[derive(Serialize, Deserialize, Clone)]
pub struct Game<'a> {
    players: [Player; 4],
    board: Board<'a>,
    bank: Bank,
    turn_number: i32,
    pub current_player_id: usize,
    current_trade_offer: Option<TradeOffer>,
    rolled_dice_this_turn: bool,
    previous_dice_roll: usize,
    players_accepted_trade_offer: [usize; 4],
    game_ended: bool,
}

#[allow(non_snake_case)]
impl Game<'_> {
    pub fn new() -> Self {
        let mut players = core::array::from_fn(|index| {
            Player::new(index)
        });
        
        for i in 0..4 {
            players[i] = Player::new(i);
        }

        let board = Board::new();
        let bank = Bank::new();

        Self {
            players,
            board,
            bank,
            turn_number: 0,
            current_player_id: 0,
            current_trade_offer: None,
            rolled_dice_this_turn: false,
            previous_dice_roll: 0,
            players_accepted_trade_offer: [0, 0, 0, 0],
            game_ended: false
        }
    }

    pub fn reset(&mut self) {
        let mut players = core::array::from_fn(|index| {
            Player::new(index)
        });
        
        for i in 0..4 {
            players[i] = Player::new(i);
        }

        let board = Board::new();
        let bank = Bank::new();

        self.players = players;
        self.board = board;
        self.bank = bank;
        self.turn_number = 0;
        self.current_player_id = 0;
        self.current_trade_offer = None;
        self.rolled_dice_this_turn = false;
        self.previous_dice_roll = 0;
        self.players_accepted_trade_offer = [0, 0, 0, 0];
        self.game_ended = false;

    }

    // Takes an action on the game. Returns the next GameState and a boolean if the action was a success.
    pub fn takeAction(&mut self, action: Action, player_id: usize) -> bool {
        // If in the initial turns, then handle the action serparately.
        if self.turn_number < 7 { 
            self.turn_number += 1;
            return self.handleInitialTurn(action, player_id);
        }

        // Check and make sure the dice have been rolled this turn.
        if self.rolled_dice_this_turn != true && action.action_type != ActionType::RollDice {
            return false;
        }

        // Check and make sure that if their is a trade offer, the action type is accept or decline trade.
        if self.current_trade_offer.is_some() && (action.action_type != ActionType::AcceptTrade || action.action_type != ActionType::DeclineTrade) {
            return false;
        }

        match action.action_type {
            ActionType::AcceptTrade => {
                // Check that there is a trade offer.
                if !self.current_trade_offer.is_some() {
                    return false;
                }

                // Check that the player has the resources to accept the trade.
                if !self.players[player_id].hasResourceCards(self.current_trade_offer.as_ref().unwrap().receiving_resources.clone()) {
                    return false;
                }

                // If the player is not the offering player, add them to the list of players who accepted the trade.
                if self.current_trade_offer.as_ref().unwrap().player_offerer_id != player_id {
                    self.current_player_id = (self.current_player_id + 1) % 4;
                    self.players_accepted_trade_offer[player_id] = player_id;
                    return true;
                }

                // Otherwise, accept the trade from the player who offered the trade.
                let player_to_accept_trade_from = action.action_metadata[0];
                if self.players_accepted_trade_offer[player_to_accept_trade_from] == 0 {
                    return false;
                }

                // Choose which player to accept the trade from, and then trade the resources.
                if !self.players[player_id].removeCardsFromHand(self.current_trade_offer.as_ref().unwrap().giving_resources.clone()) {
                    return false;
                }

                if !self.players[action.action_metadata[0]].removeCardsFromHand(self.current_trade_offer.as_ref().unwrap().receiving_resources.clone()) {
                    self.players[player_id].addResourceCards(self.current_trade_offer.as_ref().unwrap().giving_resources.clone());
                    return false;
                }

                // Trade was successful.
                return true;
            },
            ActionType::DeclineTrade => {
                if !self.current_trade_offer.is_some() {
                    return false;
                }

                self.current_player_id = (self.current_player_id + 1) % 4;
                return true;
            },
            ActionType::Discard => {
                let removed_cards = HashMap::from([
                    (ResourceCard::Ore, action.action_metadata[0]),
                    (ResourceCard::Wheat, action.action_metadata[1]),
                    (ResourceCard::Sheep, action.action_metadata[2]),
                    (ResourceCard::Brick, action.action_metadata[3]),
                    (ResourceCard::Lumber, action.action_metadata[4]),
                ]);
                let mut current_player = self.players[self.current_player_id].clone();

                // Check that the current player has the amount of resource cards to discard.
                if !current_player.hasResourceCards(removed_cards.clone()) {
                    return false;
                }

                // Get the number of cards the player has in their hand.
                let mut num_cards = 0;
                for (_, value) in current_player.resource_cards.iter() {
                    num_cards += value;
                }

                // Get the number of cards attempting to discard.
                let num_discarded_cards = 0;
                for (_, value) in removed_cards.iter() {
                    num_cards += *value;
                }

                // Checks that the previous roll was 7, the number of cards the player has in their hand is 8 or more,
                // and that the number of cards attempting to be discarded are exactly half of their hand size.
                if self.previous_dice_roll != 7 || num_cards < 8 || num_cards / 2 != num_discarded_cards {
                    return false;
                }

                current_player.removeCardsFromHand(removed_cards);
                return true;
            },
            ActionType::DrawDevelopmentCard => {
                let mut current_player = self.players[self.current_player_id].clone();
                let development_card_resources = HashMap::from([(ResourceCard::Ore, 1), (ResourceCard::Wheat, 1), (ResourceCard::Sheep, 1)]);
                
                // Check the player has the resource cards available to get a development card.
                if !current_player.hasResourceCards(development_card_resources.clone()) {
                    return false;
                }

                // Check that their are still development cards left to draw.
                let drawn_development_card = self.bank.drawDevelopmentCard();
                if drawn_development_card.is_none() {
                    return false;
                }

                // Remove the resources for the card and add it to the players hand.
                current_player.removeCardsFromHand(development_card_resources);
                current_player.addDevelopmentCard(drawn_development_card.unwrap().clone());
                self.bank.replaceResourceCard(ResourceCard::Ore, 1);
                self.bank.replaceResourceCard(ResourceCard::Wheat, 1);
                self.bank.replaceResourceCard(ResourceCard::Sheep, 1);
                return true;
            },
            ActionType::EndTurn => {
                self.players[player_id].moveDevelopmentCards();
                self.current_player_id = (self.current_player_id + 1) % 4; 
                self.rolled_dice_this_turn = false;
                return true;
            },
            ActionType::OfferTrade => {
                if self.offerPortTrade(action.clone(), player_id.clone()) {
                    return true;
                }

                // Otherwise, setup a trade for the other players.
                let giving_resources = HashMap::from([
                    (ResourceCard::Ore, action.action_metadata[0]),
                    (ResourceCard::Wheat, action.action_metadata[1]),
                    (ResourceCard::Sheep, action.action_metadata[2]),
                    (ResourceCard::Brick, action.action_metadata[3]),
                    (ResourceCard::Lumber, action.action_metadata[4])
                ]);
                let receiving_resources = HashMap::from([
                    (ResourceCard::Ore, action.action_metadata[5]),
                    (ResourceCard::Wheat, action.action_metadata[6]),
                    (ResourceCard::Sheep, action.action_metadata[7]),
                    (ResourceCard::Brick, action.action_metadata[8]),
                    (ResourceCard::Lumber, action.action_metadata[9])
                ]);

                if !self.players[player_id].hasResourceCards(giving_resources.clone()) {
                    return false;
                }

                self.current_trade_offer = Some(TradeOffer{
                    player_offerer_id: player_id,
                    giving_resources,
                    receiving_resources
                });
                self.current_player_id = (self.current_player_id + 1) % 4;
                return true;
            },
            ActionType::PlaceRobber => {
                if action.action_metadata[0] >= self.board.tiles.len() {
                    return false;
                }

                // Check if the robber is on the current tile.
                if self.board.tiles[action.action_metadata[0]].lock().unwrap().has_robber {
                    return false;
                }

                let mut has_node_owned_by_robbed_player = false;
                for node in self.board.tiles[action.action_metadata[0]].lock().unwrap().adjacent_nodes.clone() {
                    match node.lock().unwrap().building.as_mut().unwrap() {
                        Building::Settlement(_, player) => {
                            has_node_owned_by_robbed_player |= *player == action.action_metadata[1];
                        },
                        Building::City(_, player) => {
                            has_node_owned_by_robbed_player |= *player == action.action_metadata[1];
                        },
                        _ => { continue; }
                    }
                }

                if !has_node_owned_by_robbed_player {
                    return false;
                }

                // Set the robber to true on the tile and steal a card from the given player.
                self.board.tiles[action.action_metadata[0]].lock().unwrap().has_robber = true;
                let stolen_resource = self.players[action.action_metadata[1]].stealCard();
                self.players[player_id].addResourceCards(HashMap::from([(stolen_resource, 1)]));
                return true;
            },
            ActionType::PlayCity => {
                if action.action_metadata[0] >= self.board.nodes.len() {
                    return false;
                }

                if !(self.players[player_id].num_unplaced_cities > 0) {
                    return false;
                }

                let city_resources = HashMap::from([(ResourceCard::Ore, 3), (ResourceCard::Wheat, 2)]);
                if !self.players[player_id].hasResourceCards(city_resources.clone()) {
                    return false;
                }

                if !self.board.placeCity(Building::City(action.action_metadata[0], self.current_player_id)) {
                    return false;
                }

                self.players[player_id].removeCardsFromHand(city_resources);
                self.bank.replaceResourceCard(ResourceCard::Ore, 3);
                self.bank.replaceResourceCard(ResourceCard::Wheat, 2);
                self.players[player_id].num_unplaced_cities -= 1;
                self.players[player_id].num_unplaced_settlements += 1;
                self.players[player_id].victory_points += 1;
                self.players[player_id].city_placements.push(action.action_metadata[0]);
                return true;
            },
            ActionType::PlayDevelopmentCard => {
                if action.action_metadata[0] > 5 {
                    return false;
                }

                let attempted_development_card = match action.action_metadata[0] {
                    0 => DevelopmentCard::Knight,
                    1 => DevelopmentCard::Monopoly,
                    2 => DevelopmentCard::RoadBuilding,
                    3 => DevelopmentCard::VictoryPoint,
                    4 => DevelopmentCard::YearOfPlenty,
                    _ => DevelopmentCard::VictoryPoint
                };

                if attempted_development_card == DevelopmentCard::VictoryPoint {
                    return false;
                }

                if !self.players[player_id].hasDevelopmentCard(attempted_development_card.clone()) {
                    return false;
                }

                if !self.handleDevelopmentCard(action, player_id) {
                    return false;
                }
                self.players[player_id].removeDevelopmentCard(attempted_development_card);
                return true;
            },
            ActionType::PlayRoad => {
                // Check the road placement makes sense.
                if action.action_metadata[0] >= self.board.edges.len() {
                    return false;
                }

                if !(self.players[player_id].num_unplaced_roads > 0) {
                    return false;
                }

                // Check the player has the resources for a road.
                let road_resources = HashMap::from([(ResourceCard::Lumber, 1), (ResourceCard::Brick, 1)]);
                if !self.players[player_id].hasResourceCards(road_resources.clone()) {
                    return false;
                }

                // If the road cannot be played return false.
                let (placed_road, _) = self.board.placeRoad(Building::Road(action.action_metadata[0], self.current_player_id));
                if !placed_road {
                    return false;
                }

                self.players[player_id].removeCardsFromHand(road_resources);
                self.bank.replaceResourceCard(ResourceCard::Lumber, 1);
                self.bank.replaceResourceCard(ResourceCard::Brick, 1);
                self.players[player_id].num_unplaced_roads -= 1;
                self.players[player_id].road_placements.push(action.action_metadata[0]);

                // Check if the player has the longest road.
                let mut road_sizes: [usize; 4] = [0; 4];
                for player in &self.players {
                    road_sizes[player.id] = self.checkLongestRoad(player.id);
                }

                if road_sizes[player_id] < 5 {
                    return true;
                }

                for (player, road_size) in road_sizes.iter().enumerate() {
                    if player == player_id {
                        continue;
                    }

                    if road_sizes[player_id] <= *road_size {
                        return true;
                    } 

                    if self.players[player].longest_road && *road_size < road_sizes[player_id] {
                        self.players[player].longest_road = false;
                        self.players[player].victory_points -= 2;
                        self.players[player_id].longest_road = true;
                        self.players[player_id].victory_points += 2;
                        return true;
                    }
                }

                self.players[player_id].longest_road = true;
                self.players[player_id].victory_points += 2;
                return true;
            },
            ActionType::PlaySettlement => {
                // Check that the settlement placement makes sense.
                if action.action_metadata[0] >= self.board.nodes.len() {
                    return false;
                }

                if !(self.players[player_id].num_unplaced_settlements > 0) {
                    return false;
                }

                // Check the player has the resources in hand to build a settlement.
                let settlement_resources = HashMap::from([
                    (ResourceCard::Lumber, 1),
                    (ResourceCard::Brick, 1),
                    (ResourceCard::Wheat, 1),
                    (ResourceCard::Sheep, 1)
                ]);

                if !self.players[player_id].hasResourceCards(settlement_resources.clone()) {
                    return false;
                }

                if !self.board.placeSettlement(Building::Settlement(action.action_metadata[0], self.current_player_id)) {
                    return false;
                }

                self.players[player_id].removeCardsFromHand(settlement_resources);
                self.bank.replaceResourceCard(ResourceCard::Lumber, 1);
                self.bank.replaceResourceCard(ResourceCard::Wheat, 1);
                self.bank.replaceResourceCard(ResourceCard::Brick, 1);
                self.bank.replaceResourceCard(ResourceCard::Sheep, 1);
                self.players[player_id].victory_points += 1;
                self.players[player_id].num_unplaced_settlements -= 1;
                self.players[player_id].settlement_placements.push(action.action_metadata[0]);
                return true;
            },
            ActionType::RollDice => {
                if self.rolled_dice_this_turn {
                    return false;
                }

                // Roll the dice and produce on the relevant tiles.
                let roll_1 = rand::thread_rng().gen_range(1..6);
                let roll_2 = rand::thread_rng().gen_range(1..6);

                self.previous_dice_roll = roll_1 + roll_2;
                self.rolled_dice_this_turn = true;
                if self.previous_dice_roll == 7 {
                    return true;
                }
                self.produceDiceRoll(self.previous_dice_roll);
                return true;
            }
        }
    }

    // Offers a port trade, returns whether it was successful or not.
    fn offerPortTrade(&mut self, action: Action, player_id: usize) -> bool {
        // Check if all the trade resources offered are less than 2; no port trade can occur, return false;
        if action.action_metadata[0..5].iter().map(|num_trade| { *num_trade < 2 }).all(|less_2| less_2) {
            return false;
        }

        // Check if the receiving resources add to greater than one; if so, return false.
        if action.action_metadata[5..11].iter().fold(0, |acc, x| x + acc) > 1 {
            return false;
        }

        // Check if offering more than 1 resource type; if so, return false.
        if action.action_metadata[0..5].iter().filter(|resource| **resource > 0).collect::<Vec<&usize>>().len() > 1 {
            return false;
        }

        // Check if one of the nodes the player has a settlement or city on is a port node.
        let port_nodes = [0, 1, 3, 4, 10, 11, 15, 16, 26, 32, 33, 38, 42, 46, 47, 49, 51, 52];
        let player_has_building_on_port: bool = self.players[player_id].settlement_placements.iter().filter(|placement| {
            port_nodes.contains(placement)
        }).collect::<Vec<&usize>>().len() > 0 || self.players[player_id].city_placements.iter().filter(|placement| {
            port_nodes.contains(placement)
        }).collect::<Vec<&usize>>().len() > 0;

        if !player_has_building_on_port {
            return false;
        }

        let cloned_player = self.players[player_id].clone();
        let mut settlements_on_ports = cloned_player.settlement_placements.iter().filter(|placement| {
            port_nodes.contains(placement)
        }).collect::<Vec<&usize>>().clone();
        let mut cities_on_ports = cloned_player.city_placements.iter().filter(|placement| {
            port_nodes.contains(placement)
        }).collect::<Vec<&usize>>().clone();
        settlements_on_ports.append(&mut cities_on_ports);

        // Helper variable in case resources get traded.
        let giving_resources = HashMap::from([
            (ResourceCard::Ore, action.action_metadata[0]),
            (ResourceCard::Wheat, action.action_metadata[1]),
            (ResourceCard::Sheep, action.action_metadata[2]),
            (ResourceCard::Brick, action.action_metadata[3]),
            (ResourceCard::Lumber, action.action_metadata[4])
        ]);

        // Check whether the trade is valid.
        for (port_num, nodes) in self.board.port_node_mapping.clone().iter().enumerate() {
            let node1 = nodes.lock().unwrap().0;
            let node2 = nodes.lock().unwrap().1;

            let has_building_on_this_port = settlements_on_ports.iter().filter(
                |node| { ***node == node1 || ***node == node2 }
            ).collect::<Vec<&&usize>>().len() > 0;

            if !has_building_on_this_port {
                continue;
            }

            // Do the trade if valid.
            match *self.board.ports[port_num].lock().unwrap() {
                Port::ThreeToOne => {
                    let offering_3_trade = action.action_metadata[0..5].iter()
                        .filter(|num_to_trade| **num_to_trade == 3).collect::<Vec<&usize>>().len() > 0;

                    if !offering_3_trade {
                        continue;
                    }

                    if action.action_metadata[5] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Ore, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Ore);
                        return true;
                    } else if action.action_metadata[6] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Wheat, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Wheat);
                        return true;
                    } else if action.action_metadata[7] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Sheep, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Sheep);
                        return true;
                    } else if action.action_metadata[8] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Brick, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Brick);
                        return true;
                    } else if action.action_metadata[9] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Lumber, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Lumber);
                        return true;
                    }
                },
                Port::Brick => {
                    let offering_2_trade_brick = action.action_metadata[3] == 2;

                    if !offering_2_trade_brick {
                        continue;
                    }

                    if action.action_metadata[5] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Ore, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Ore);
                        return true;
                    } else if action.action_metadata[6] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Wheat, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Wheat);
                        return true;
                    } else if action.action_metadata[7] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Sheep, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Sheep);
                        return true;
                    } else if action.action_metadata[8] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Brick, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Brick);
                        return true;
                    } else if action.action_metadata[9] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Lumber, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Lumber);
                        return true;
                    }
                },
                Port::Lumber => {
                    let offering_2_trade_lumber = action.action_metadata[4] == 2;
                    
                    if !offering_2_trade_lumber {
                        continue;
                    }

                    if action.action_metadata[5] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Ore, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Ore);
                        return true;
                    } else if action.action_metadata[6] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Wheat, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Wheat);
                        return true;
                    } else if action.action_metadata[7] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Sheep, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Sheep);
                        return true;
                    } else if action.action_metadata[8] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Brick, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Brick);
                        return true;
                    } else if action.action_metadata[9] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Lumber, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Lumber);
                        return true;
                    }
                },
                Port::Ore => {
                    let offering_2_trade_ore = action.action_metadata[0] == 2;
                    
                    if !offering_2_trade_ore {
                        continue;
                    }

                    if action.action_metadata[5] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Ore, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Ore);
                        return true;
                    } else if action.action_metadata[6] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Wheat, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Wheat);
                        return true;
                    } else if action.action_metadata[7] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Sheep, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Sheep);
                        return true;
                    } else if action.action_metadata[8] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Brick, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Brick);
                        return true;
                    } else if action.action_metadata[9] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Lumber, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Lumber);
                        return true;
                    }
                },
                Port::Sheep => {
                    let offering_2_trade_sheep = action.action_metadata[2] == 2;
                    
                    if !offering_2_trade_sheep {
                        continue;
                    }

                    if action.action_metadata[5] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Ore, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Ore);
                        return true;
                    } else if action.action_metadata[6] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Wheat, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Wheat);
                        return true;
                    } else if action.action_metadata[7] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Sheep, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Sheep);
                        return true;
                    } else if action.action_metadata[8] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Brick, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Brick);
                        return true;
                    } else if action.action_metadata[9] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Lumber, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Lumber);
                        return true;
                    }

                },
                Port::Wheat => {
                    let offering_2_trade_wheat = action.action_metadata[1] == 2;
                    
                    if !offering_2_trade_wheat {
                        continue;
                    }

                    if action.action_metadata[5] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Ore, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Ore);
                        return true;
                    } else if action.action_metadata[6] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Wheat, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Wheat);
                        return true;
                    } else if action.action_metadata[7] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Sheep, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Sheep);
                        return true;
                    } else if action.action_metadata[8] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Brick, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Brick);
                        return true;
                    } else if action.action_metadata[9] > 0 {
                        if !self.bank.drawNumberOfResourceCards(ResourceCard::Lumber, 1) {
                            return false;
                        }
                        self.players[player_id].borrow_mut().removeCardsFromHand(giving_resources.clone());
                        self.players[player_id].addResourceCard(ResourceCard::Lumber);
                        return true;
                    }
                }
            }
        }

        false
    }

    fn checkLongestRoad(&self, player_id: usize) -> usize {
        // Check if the player has longest road
        let mut road_sizes: Vec<usize> = vec![];
        self.players[player_id].road_placements.iter().for_each(|road_placement| {
            let adjacent_roads = self.board.edges[*road_placement].lock().unwrap().adjacent_nodes.iter()
                .map(|node| {
                    node.lock().unwrap().adjacent_edges.clone()
                })
                .flatten()
                .filter(|road| { road.lock().unwrap().position != *road_placement })
                .filter(|road| {
                    match road.lock().unwrap().building {
                        Some(Building::Road(_, player)) => {
                            if player != player_id {
                                return false;
                            }
                            return true;
                        },
                        _ => false
                    }
                })
                .map(|road| {
                    road.lock().unwrap().position
                })
                .collect();

            road_sizes.push(self.checkRoadSize(player_id, 1, vec![*road_placement], adjacent_roads));
        });

        return *road_sizes.iter().max().unwrap();
    }

    fn checkRoadSize(&self, player_id: usize, road_length: usize, mut already_checked_roads: Vec<usize>, unchecked_roads: Vec<usize>) -> usize {
        let mut road_sizes: Vec<usize> = vec![];
        for road_placement in unchecked_roads {
            already_checked_roads.push(road_placement);
            let adjacent_roads: Vec<usize> = self.board.edges[road_placement].lock().unwrap().adjacent_nodes.iter()
                .map(|node| {
                    node.lock().unwrap().adjacent_edges.clone()
                })
                .flatten()
                .filter(|road| { !already_checked_roads.contains(&road.lock().unwrap().position) })
                .filter(|road| {
                    match road.lock().unwrap().building {
                        Some(Building::Road(_, player)) => {
                            if player != player_id {
                                return false;
                            }
                            return true;
                        }
                        _ => false
                    }
                })
                .map(|road| {
                    road.lock().unwrap().position
                })
                .collect();
            road_sizes.push(self.checkRoadSize(player_id, road_length + 1, already_checked_roads.clone(), adjacent_roads));
        }

        match road_sizes.iter().max() {
            Some(val) => {
                road_length + *val
            },
            None => road_length
        }
    }

    fn produceDiceRoll(&mut self, dice_roll: usize) {
        let producing_tiles = self.board.tiles.iter().filter(|tile| tile.lock().unwrap().chit == dice_roll as i32);

        // <PlayerID, resources to draw>
        let mut resources_to_produce: HashMap<ResourceCard, usize> = HashMap::from([
            (ResourceCard::Ore, 0),
            (ResourceCard::Wheat, 0),
            (ResourceCard::Sheep, 0),
            (ResourceCard::Brick, 0),
            (ResourceCard::Lumber, 0),
        ]);

        let mut player_resource_production: HashMap<usize, HashMap<ResourceCard, usize>> = HashMap::from([
            (0, resources_to_produce.clone()),
            (1, resources_to_produce.clone()),
            (2, resources_to_produce.clone()),
            (3, resources_to_produce.clone())
        ]);

        for tile in producing_tiles {
            let cur_tile = tile.lock().unwrap();
            let producing_nodes = cur_tile.adjacent_nodes.iter().filter(|node| {
                node.lock().unwrap().hasBuilding()
            });

            // todo!("Refactor this to draw cards from the bank.");

            for node in producing_nodes {
                match node.lock().unwrap().building.as_ref().unwrap() {
                    Building::City(_, player_id) => {
                        match cur_tile.terrain {
                            Terrain::Fields => {
                                let resource_to_produce = ResourceCard::Wheat;
                                let new_val = player_resource_production.get(&player_id).unwrap().get(&resource_to_produce).unwrap() + 2;
                                player_resource_production.get_mut(player_id).unwrap().insert(resource_to_produce, new_val);
                                resources_to_produce.insert(resource_to_produce, *resources_to_produce.get(&resource_to_produce).unwrap() + 2);
                            },
                            Terrain::Forest => {
                                let resource_to_produce = ResourceCard::Lumber;
                                let new_val = player_resource_production.get(&player_id).unwrap().get(&resource_to_produce).unwrap() + 2;
                                player_resource_production.get_mut(player_id).unwrap().insert(resource_to_produce, new_val);
                                resources_to_produce.insert(resource_to_produce, *resources_to_produce.get(&resource_to_produce).unwrap() + 2);
                            },
                            Terrain::Hills => {
                                let resource_to_produce = ResourceCard::Brick;
                                let new_val = player_resource_production.get(&player_id).unwrap().get(&resource_to_produce).unwrap() + 2;
                                player_resource_production.get_mut(player_id).unwrap().insert(resource_to_produce, new_val);
                                resources_to_produce.insert(resource_to_produce, *resources_to_produce.get(&resource_to_produce).unwrap() + 2);
                            },
                            Terrain::Plains => {
                                let resource_to_produce = ResourceCard::Sheep;
                                let new_val = player_resource_production.get(&player_id).unwrap().get(&resource_to_produce).unwrap() + 2;
                                player_resource_production.get_mut(player_id).unwrap().insert(resource_to_produce, new_val);
                                resources_to_produce.insert(resource_to_produce, *resources_to_produce.get(&resource_to_produce).unwrap() + 2);
                            },
                            Terrain::Mountains => {
                                let resource_to_produce = ResourceCard::Ore;
                                let new_val = player_resource_production.get(&player_id).unwrap().get(&resource_to_produce).unwrap() + 2;
                                player_resource_production.get_mut(player_id).unwrap().insert(resource_to_produce, new_val);
                                resources_to_produce.insert(resource_to_produce, *resources_to_produce.get(&resource_to_produce).unwrap() + 2);
                            },
                            Terrain::Desert => { continue; }
                        }
                    },
                    Building::Settlement(_, player_id) => {
                        match cur_tile.terrain {
                            Terrain::Fields => {
                                let resource_to_produce = ResourceCard::Wheat;
                                let new_val = player_resource_production.get(&player_id).unwrap().get(&resource_to_produce).unwrap() + 1;
                                player_resource_production.get_mut(player_id).unwrap().insert(resource_to_produce, new_val);
                                resources_to_produce.insert(resource_to_produce, *resources_to_produce.get(&resource_to_produce).unwrap() + 1);
                            },
                            Terrain::Forest => {
                                let resource_to_produce = ResourceCard::Lumber;
                                let new_val = player_resource_production.get(&player_id).unwrap().get(&resource_to_produce).unwrap() + 1;
                                player_resource_production.get_mut(player_id).unwrap().insert(resource_to_produce, new_val);
                                resources_to_produce.insert(resource_to_produce, *resources_to_produce.get(&resource_to_produce).unwrap() + 1);
                            },
                            Terrain::Hills => {
                                let resource_to_produce = ResourceCard::Brick;
                                let new_val = player_resource_production.get(&player_id).unwrap().get(&resource_to_produce).unwrap() + 1;
                                player_resource_production.get_mut(player_id).unwrap().insert(resource_to_produce, new_val);
                                resources_to_produce.insert(resource_to_produce, *resources_to_produce.get(&resource_to_produce).unwrap() + 1);
                            },
                            Terrain::Plains => {
                                let resource_to_produce = ResourceCard::Sheep;
                                let new_val = player_resource_production.get(&player_id).unwrap().get(&resource_to_produce).unwrap() + 1;
                                player_resource_production.get_mut(player_id).unwrap().insert(resource_to_produce, new_val);
                                resources_to_produce.insert(resource_to_produce, *resources_to_produce.get(&resource_to_produce).unwrap() + 1);
                            },
                            Terrain::Mountains => {
                                let resource_to_produce = ResourceCard::Ore;
                                let new_val = player_resource_production.get(&player_id).unwrap().get(&resource_to_produce).unwrap() + 1;
                                player_resource_production.get_mut(player_id).unwrap().insert(resource_to_produce, new_val);
                                resources_to_produce.insert(resource_to_produce, *resources_to_produce.get(&resource_to_produce).unwrap() + 1);
                            },
                            Terrain::Desert => { continue; }
                        }
                    },
                    _ => { continue; }
                }
            }
        }

        // Check if the bank has enough to produce the resources.
        let mut can_produce: HashMap<ResourceCard, bool> = HashMap::from([
            (ResourceCard::Ore, false),
            (ResourceCard::Wheat, false),
            (ResourceCard::Sheep, false),
            (ResourceCard::Brick, false),
            (ResourceCard::Lumber, false),
        ]);

        // Check which resources are produceable or not.
        for (resource, amount_to_produce) in resources_to_produce {
            if self.bank.amountOfResource(resource) < amount_to_produce {

                // Check if only one player is attempting to produce that resource.
                let mut num_players_trying_to_produce_resource = 0;
                for (_, resources) in &player_resource_production {
                    if *resources.get(&resource).unwrap() > 0 {
                        num_players_trying_to_produce_resource += 1;
                    }
                }

                if num_players_trying_to_produce_resource > 1 {
                    can_produce.insert(resource, false);
                }
            }
        }

        // Take the cards from the bank and give them to the players.
        for (resource, producable) in can_produce {
            if producable {
                for (player, resources) in &player_resource_production {
                    // Remove the cards from the bank.
                    if self.bank.drawNumberOfResourceCards(resource, *resources.get(&resource).unwrap()) {
                        // Give the cards to the player.
                        self.players[*player].addResourceCardAmount(resource, *resources.get(&resource).unwrap());
                    } else {
                        self.players[*player].addResourceCardAmount(resource, self.bank.amountOfResource(resource));
                        self.bank.drawNumberOfResourceCards(resource, self.bank.amountOfResource(resource));
                    }

                }
            }
        }
    }

    fn handleDevelopmentCard(&mut self, action: Action, player_id: usize) -> bool {
        match action.action_metadata[0] {
            // Robber development card.
            0 => {
                if action.action_metadata[1] >= self.board.tiles.len() {
                    return false;
                }

                // Check if the robber is on the current tile.
                if self.board.tiles[action.action_metadata[1]].lock().unwrap().has_robber {
                    return false;
                }

                let mut has_node_owned_by_robbed_player = false;
                for node in self.board.tiles[action.action_metadata[1]].lock().unwrap().adjacent_nodes.clone() {
                    match node.lock().unwrap().building.as_mut().unwrap() {
                        Building::Settlement(_, player) => {
                            has_node_owned_by_robbed_player |= *player == action.action_metadata[2];
                        },
                        Building::City(_, player) => {
                            has_node_owned_by_robbed_player |= *player == action.action_metadata[2];
                        },
                        _ => { continue; }
                    }
                }

                if !has_node_owned_by_robbed_player {
                    return false;
                }

                // Set the robber to true on the tile and steal a card from the given player.
                self.board.tiles[action.action_metadata[0]].lock().unwrap().has_robber = true;
                let stolen_resource = self.players[action.action_metadata[1]].stealCard();
                self.players[player_id].addResourceCards(HashMap::from([(stolen_resource, 1)]));
                self.players[player_id].num_knights_played += 1;

                // Check if the player has largest army.
                if !self.players[player_id].num_knights_played >= 3 {
                    return true;
                }

                for player in &self.players[0..4] {
                    if player.id == player_id { continue; }
                    if !(self.players[player_id].num_knights_played > player.num_knights_played){
                        return true;
                    }
                }

                for player in self.players[0..4].as_mut() {
                    if player.largest_army {
                        player.largest_army = false;
                        player.victory_points -= 2;
                    }
                }

                self.players[player_id].largest_army = true;
                self.players[player_id].victory_points += 2;
                return true;
            },
            1 => {
                // Make sure that the resource makes sense
                if action.action_metadata[1] >= 5 {
                    return false;
                }
                let mut amount_to_add: usize = 0;
                for mut player in self.players.clone() {
                    if player.id == player_id {
                        continue;
                    }
                    match action.action_metadata[1] {
                        0 => { amount_to_add += player.removeAllResourcesFromHand(ResourceCard::Ore); },
                        1 => { amount_to_add += player.removeAllResourcesFromHand(ResourceCard::Wheat); },
                        2 => { amount_to_add += player.removeAllResourcesFromHand(ResourceCard::Sheep); },
                        3 => { amount_to_add += player.removeAllResourcesFromHand(ResourceCard::Brick); },
                        4 => { amount_to_add += player.removeAllResourcesFromHand(ResourceCard::Lumber); },
                        _ => { return false; }
                    }
                }

                // Add the resource to the players hand.
                match action.action_metadata[1] {
                    0 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Ore, amount_to_add)])); },
                    1 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Wheat, amount_to_add)])); },
                    2 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Sheep, amount_to_add)])); },
                    3 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Brick, amount_to_add)])); },
                    4 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Lumber, amount_to_add)])); },
                    _ => { return false; }
                }
                

                return true
            },
            2 => {
                if action.action_metadata[1] >= self.board.edges.len() || action.action_metadata[2] >= self.board.edges.len() {
                    return false;
                }

                let road_1 = Building::Road(action.action_metadata[1], player_id);
                let road_2 = Building::Road(action.action_metadata[2], player_id);

                let (placed_road1, already_road1) = self.board.placeRoad(road_1.clone());
                let (placed_road2, already_road2) = self.board.placeRoad(road_2.clone());

                if !placed_road1 || !placed_road2 {
                    if !already_road1 {
                        self.board.removeRoad(road_1);
                    }
                    if !already_road2 {
                        self.board.removeRoad(road_2);
                    }
                    return false;
                }

                return true;
            },
            3 => {
                return false;
            },
            4 => {
                if action.action_metadata[1] >= 5 || action.action_metadata[2] >= 5 {
                    return false;
                }
                match action.action_metadata[1] {
                    0 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Ore, 1)]));},
                    1 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Wheat, 1)]));},
                    2 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Sheep, 1)]));},
                    3 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Brick, 1)]));},
                    4 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Lumber, 1)]));},
                    _ => {return false; }
                }
                match action.action_metadata[2] {
                    0 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Ore, 1)]));},
                    1 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Wheat, 1)]));},
                    2 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Sheep, 1)]));},
                    3 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Brick, 1)]));},
                    4 => { self.players[player_id].addResourceCards(HashMap::from([(ResourceCard::Lumber, 1)]));},
                    _ => {return false; }
                }

                return true;
            }
            _ => { return false; }
        }
    }

    fn handleInitialTurn(&mut self, action: Action, player_id: usize) -> bool {
        match action.action_type {
            ActionType::PlaySettlement => {
                let new_settlement = Building::Settlement(
                    action.action_metadata[0],
                    player_id
                );
                if self.board.placeInitialSettlement(new_settlement) {
                    self.players[player_id].num_unplaced_settlements -= 1;
                    self.players[player_id].settlement_placements.push(action.action_metadata[0]);
                    return true;
                } else {
                    return false;
                }
            },
            ActionType::PlayRoad => {
                let new_road = Building::Settlement(
                    action.action_metadata[0],
                    player_id
                );
                if self.board.placeInitialRoad(new_road) {
                    self.players[player_id].num_unplaced_roads -= 1;
                    self.players[player_id].road_placements.push(action.action_metadata[0]);
                    self.current_player_id = (self.current_player_id + 1) % 4;
                    return true;
                } else { 
                    return false;
                }
            },
            _ => false
        }
    }
}