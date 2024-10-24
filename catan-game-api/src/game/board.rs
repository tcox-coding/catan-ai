use rand::prelude::*;
use std::io;
use std::fs::File;
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

use crate::game::port::Port;
use crate::game::node::Node;
use crate::game::edge::Edge;
use crate::game::tile::Tile;
use crate::game::building::Building;
use crate::game::terrain::Terrain;

#[derive(Serialize, Deserialize, Clone)]
pub struct Board<'a> {
    pub ports: [Arc<Mutex<Port>>; 9],
    pub tiles: [Arc<Mutex<Tile<'a>>>; 19],
    pub nodes: Vec<Arc<Mutex<Node<'a>>>>,
    pub edges: Vec<Arc<Mutex<Edge<'a>>>>,
    pub port_node_mapping: Vec<Arc<Mutex<(usize, usize)>>>,
}

#[allow(non_snake_case)]
impl Board<'_> {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        
        // Initialize nodes and edges.
        let nodes: Vec<Arc<Mutex<Node<'_>>>> = core::array::from_fn::<_, 54, _>(|index| {
            Arc::new(Mutex::new(Node::new(index)))
        }).to_vec();
        let edges: Vec<Arc<Mutex<Edge<'_>>>> = core::array::from_fn::<_, 72, _>(|index| {
            Arc::new(Mutex::new(Edge::new(index)))
        }).to_vec();
        // Map nodes and edges together.
        if let Ok(lines) = read_lines("./node_edge_mapping.txt") {
            for (i, line) in lines.flatten().enumerate() {
                if i == 0 { continue; }
                let parts: Vec<&str> = line.split(": ").collect();
                let node = parts[0];
                let edge: Vec<&str> = parts[1].split(", ").collect();
                for e in edge {
                    // println!("{}", e.parse::<usize>().unwrap() - 1);
                    let cur_edge = edges[e.parse::<usize>().unwrap() - 1].clone();
                    let cur_node = nodes[node.parse::<usize>().unwrap() - 1].clone();
                    cur_node.lock().unwrap().adjacent_edges.push(
                        cur_edge.clone()
                    );
                    cur_edge.lock().unwrap().adjacent_nodes.push(
                        cur_node.clone()
                    );
                }
            }
        }

        // Initialize port nodes.
        let mut ports = core::array::from_fn(|i| {
            if i < 4 {
                Arc::new(Mutex::new(Port::ThreeToOne))
            } else if i < 5 {
                Arc::new(Mutex::new(Port::Lumber))
            } else if i < 6 {
                Arc::new(Mutex::new(Port::Ore))
            } else if i < 7 {
                Arc::new(Mutex::new(Port::Wheat))
            } else if i < 8 {
                Arc::new(Mutex::new(Port::Sheep))
            } else if i < 9 {
                Arc::new(Mutex::new(Port::Brick))
            } else {
                Arc::new(Mutex::new(Port::ThreeToOne))
            }
        });
        ports.shuffle(&mut rng);

        // Initialize Tiles
        let mut tiles = core::array::from_fn(|i| {
            if i < 4 {
                Arc::new(Mutex::new(Tile::new(0, Terrain::Forest, 0)))
            } else if i < 8 {
                Arc::new(Mutex::new(Tile::new(0, Terrain::Plains, 0)))
            } else if i < 12 {
                Arc::new(Mutex::new(Tile::new(0, Terrain::Fields, 0)))
            } else if i < 15 {
                Arc::new(Mutex::new(Tile::new(0, Terrain::Mountains, 0)))
            } else if i < 18 {
                Arc::new(Mutex::new(Tile::new(0, Terrain::Hills, 0)))
            } else {
                Arc::new(Mutex::new(Tile::new(0, Terrain::Desert, 0)))
            }
        });
        tiles.shuffle(&mut rng);
        for (i, tile) in tiles.iter().enumerate() {
            tile.lock().unwrap().index = i;
        }
        // Map nodes and tiles together.
        if let Ok(lines) = read_lines("./node_tile_mapping.txt") {
            for (i, line) in lines.flatten().enumerate() {
                if i == 0 { continue; }
                let parts: Vec<&str> = line.split(": ").collect();
                let tile = parts[0];
                let node: Vec<&str> = parts[1].split(", ").collect();
                for n in node {
                    let cur_node = nodes[n.parse::<usize>().unwrap() - 1].clone();
                    let cur_tile = tiles[tile.parse::<usize>().unwrap() - 1].clone();
                    cur_tile.lock().unwrap().adjacent_nodes.push(
                        cur_node.clone()
                    );
                    
                    cur_node.lock().unwrap().adjacent_tiles.push(
                        cur_tile.clone()
                    );
                }
            }
        }

        let port_node_mapping: Vec<Arc<Mutex<(usize, usize)>>> = vec![
            Arc::new(Mutex::new((0, 3))),
            Arc::new(Mutex::new((1, 5))),
            Arc::new(Mutex::new((10, 15))),
            Arc::new(Mutex::new((11, 16))),
            Arc::new(Mutex::new((26, 32))),
            Arc::new(Mutex::new((33, 38))),
            Arc::new(Mutex::new((42, 46))),
            Arc::new(Mutex::new((47, 51))),
            Arc::new(Mutex::new((49, 52))),
        ];
        // println!("Successfully created new board.");
        Board {
            nodes,
            edges,
            ports,
            tiles,
            port_node_mapping
        }
    }

    // Attempts to place a road. Returns whether the road was placed, and whether there was already a road there.
    // (road_placed, road_already_there)
    pub fn placeRoad(&self, road: Building) -> (bool, bool) {
        // Check that the placement is a road.
        if let Building::Road(position, player) = road {
            // Check to make sure no other road is placed on the current road attempted placement.
            let mut attempted_placement_edge = self.edges[position].lock().unwrap();
            match attempted_placement_edge.building {
                Some(_) => return (false, true),
                None => {
                    // Collect the edges most adjacent to the current edge.
                    let current_nodes = attempted_placement_edge.adjacent_nodes.clone();
                    let mut current_edges: Vec<Arc<Mutex<Edge<'_>>>> = vec![];
                    for node in current_nodes {
                        for edge in node.lock().unwrap().adjacent_edges.clone() {
                            if edge.lock().unwrap().position != attempted_placement_edge.position {
                                current_edges.push(edge);
                            }
                        }
                    }

                    for edge in current_edges {
                        let current_edge = &edge.lock().unwrap().building;
                        if current_edge.is_some() {
                            if let Building::Road(_, play) = current_edge.as_ref().unwrap() {
                                if *play == player {
                                    // Place the road
                                    attempted_placement_edge.building = Some(road);
                                    return (true, false)
                                }
                            }
                        }
                    }
                    return (false, false);
                }
            }
        }
        (false, false)
    }

    // Attempts to place a settlement
    pub fn placeSettlement(&self, settlement: Building) -> bool {
        match settlement {
            Building::Settlement(settlement_placement, settlement_player) => {
                // Check to make sure the attemped settlement placement is 
                let mut attempted_placement_node = self.nodes[settlement_placement].lock().unwrap();
                if attempted_placement_node.hasBuilding() { return false; }

                // Check to make sure player is not near any other cities / settlement (1 road away),
                // and the player has a road to the attempted settlement placement.
                let mut has_nearby_road = false;
                for edge in attempted_placement_node.adjacent_edges.clone() {
                    let cur_edge = edge.lock().unwrap(); 
                    if cur_edge.hasRoad() {
                        let Building::Road(_, play) = cur_edge.building.as_ref().unwrap() else { panic!("Non-road building on an edge."); };
                        if settlement_player == *play {
                            has_nearby_road = true;
                        }
                    }
                    for node in edge.lock().unwrap().adjacent_nodes.clone() {
                        if node.lock().unwrap().hasBuilding() {
                            return false;
                        }
                    }
                }
                if !has_nearby_road { return false; }

                // Place the settlement
                attempted_placement_node.building = Some(settlement);

                return true;
            },
            _ => { return false; }
        }
    }

    // Attempts to place a city
    pub fn placeCity(&self, city: Building) -> bool {
        // Check if the player has a settlement on the attempted node.
        match city {
            Building::City(position, player) => {
                let mut current_node = self.nodes[position].lock().unwrap();
                if current_node.hasBuilding() {
                    // Check if the position and the player of the new city are correct.
                    let Building::Settlement(pos, play) = current_node.building.as_ref().unwrap() else { return false; };
                    if *pos != position && *play != player { return false; }

                    // Place the city
                    current_node.building = Some(city);
                    return true;
                }
                return true;
            },
            _ => { return false; }
        }
    }

    // Attempts to place a settlement (beginning settlement).
    pub fn placeInitialSettlement(&self, settlement: Building) -> bool {
        match settlement {
            Building::Settlement(position, _) => {
                if position > 53 {
                    return false;
                }
                let current_node = &self.nodes[position];
                if current_node.lock().unwrap().hasBuilding() {
                    return false;
                }

                // Check to make sure that settlement is at least 2 away from another settlement.
                for edge in current_node.lock().unwrap().adjacent_edges.clone() {
                    for node in edge.lock().as_ref().unwrap().adjacent_nodes.clone() {
                        if node.try_lock().is_err() {
                            continue;
                        }
                        if node.lock().unwrap().hasBuilding() {
                            return false;
                        }
                    }
                }

                current_node.lock().unwrap().building = Some(settlement);
                return true;
            },
            _ => { return false; }
        }
    }

    // Attempts to place a road (initial road).
    pub fn placeInitialRoad(&self, road: Building) -> bool {
        match road {
            Building::Road(position, player) => {
                let mut current_edge = self.edges[position].lock().unwrap();

                // Ensure the current edge doesn't have a road.
                if current_edge.hasRoad() { return false; }

                // Ensure there is a settlement near attempted placement.
                let mut adjacent_node_has_settlement = false;
                for node in current_edge.adjacent_nodes.clone() {
                    let current_node = node.lock().unwrap(); 
                    if current_node.hasBuilding() {
                        let Building::Settlement(_, play) = current_node.building.as_ref().unwrap() else { continue; };
                        if *play == player {
                            adjacent_node_has_settlement = true;
                        }
                    }
                }
                if !adjacent_node_has_settlement { return false; }

                // Add the road to the board.
                current_edge.building = Some(road);
                return true;
            },
            _ => { return false; }
        }
    }


    // Removes a road from the board.
    pub fn removeRoad(&self, road: Building) -> bool {
        match road {
            Building::Road(pos, _) => {
                self.edges[pos].lock().unwrap().building = None;
                return true;
            },
            _ => { return false; }
        }
    }
}


        
// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines(filename: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}