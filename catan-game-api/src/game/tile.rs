use crate::game::terrain::Terrain;
use crate::game::node::Node;
use std::sync::{Arc, Mutex};

use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize)]
pub struct Tile<'a> {
    pub index: usize,
    pub terrain: Terrain,
    pub chit: i32,
    #[serde(skip)]
    pub adjacent_nodes: Vec<Arc<Mutex<Node<'a>>>>,
    pub has_robber: bool,
}

impl <'a> Tile<'a> {
    pub fn new(index: usize, terrain: Terrain, chit: i32) -> Self {
        if terrain == Terrain::Desert {
            return Self {
                index,
                terrain,
                chit,
                adjacent_nodes: vec![],
                has_robber: true,
            }
        }
        Self {
            index,
            terrain,
            chit,
            adjacent_nodes: vec![],
            has_robber: false,
        }
    }
}