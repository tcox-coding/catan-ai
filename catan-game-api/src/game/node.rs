use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

use crate::game::edge::Edge;
use crate::game::tile::Tile;
use crate::game::building::Building;

#[derive(Serialize, Deserialize)]
pub struct Node<'a> {
    pub position: usize,
    pub building: Option<Building>,
    #[serde(skip)]
    pub adjacent_edges: Vec<Arc<Mutex<Edge<'a>>>>,
    #[serde(skip)]
    pub adjacent_tiles: Vec<Arc<Mutex<Tile<'a>>>>
}

#[allow(non_snake_case)]
impl Node<'_> {
    pub fn new<'a>(position: usize) -> Self {
        Node {
            position,
            building: None,
            adjacent_edges: vec![],
            adjacent_tiles: vec![]
        }
    }

    // Returns whether the node has a building or not.
    pub fn hasBuilding(&self) -> bool {
        match self.building {
            Some(_) => true,
            None => false
        }
    }
}