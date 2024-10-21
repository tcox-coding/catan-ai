use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

use crate::game::node::Node;
use crate::game::building::Building;

#[derive(Serialize, Deserialize)]
pub struct Edge<'a> {
    pub position: usize,
    pub building: Option<Building>,
    #[serde(skip)]
    pub adjacent_nodes: Vec<Arc<Mutex<Node<'a>>>>
}

#[allow(non_snake_case)]
impl <'b> Edge<'b> {
    pub fn new(position: usize) -> Self {
        Edge {
            position: position,
            building: None,
            adjacent_nodes: vec![]
        }
    }
    
    pub fn hasRoad(&self) -> bool{
        match self.building {
            Some(_) => true,
            _ => false
        }
    }
}