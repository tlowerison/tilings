pub mod edge_rules;
pub mod vertex_rules;

use std::collections::HashMap;
use crate::common::*;
use crate::tile::*;

pub struct Hash(pub(crate) u64);

impl Eq for Hash {}

impl std::hash::Hash for Hash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialEq for Hash {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

pub struct Neighbor {
    hash: Hash,
    proto_tile: ProtoTile,
}

pub trait Tiling {
    fn get_neighbor(self: &Self, hash: Hash, edge: usize, point: Point) -> Neighbor;
}

pub struct Cell {
    pub index: usize,
    pub proto_tile: ProtoTile,
    neighbors: HashMap<Hash, usize>,
}

impl Cell {
    pub fn add_neighbor<'a, T: Tiling>(&mut self, patch: &'a Patch<T>, hash: Hash, neighbor_index: usize) {
        if neighbor_index >= patch.size() {
            panic!("failed to add neighbor for Cell {}: Cell {} does not exist in Patch", self.index, neighbor_index);
        }
        self.neighbors.insert(hash, neighbor_index);
    }

    pub fn get_neighbors(&self) -> &HashMap<Hash, usize> {
        &self.neighbors
    }
}

pub struct Patch<T: Tiling> {
    cells: Vec<Cell>,
    tiling: T,
}

impl<T: Tiling> Patch<T> {
    fn step(&mut self, cell: usize, edge: usize, point: Point) {
        if cell >= self.size() {
            panic!("Patch of size {} does not have cell {}", self.size(), cell);
        }
    }

    pub fn size(&self) -> usize {
        self.cells.len()
    }
}
