use itertools::*;
use nohash_hasher::IntMap;
use rayon::prelude::*;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    hash::{Hash, Hasher},
};
use crate::interval::*;
use crate::tile::*;
use crate::tiling::Tiling;

pub struct Cell<'a> {
    pub proto_tile: ProtoTile,
    pub neighbors: Vec<&'a ProtoTile>,
    pub state: &'a State,
    pub mask: Interval,
}

pub struct Simulation<'a> {
    pub tiling: Tiling,
    pub cells: BTreeMap<u64, Cell<'a>>,
    pub generation: u64,
    pub states: [Vec<State>; 2],
    pub allowed_states: HashMap<ProtoTile, u8>,
}

pub struct State {
    pub value: u64,
    pub positioned_intervals: BTreeSet<PositionedInterval>,
    pub sized_intervals: BTreeSet<SizedInterval>,
}

impl State {
    pub fn new() -> State {
        let mut positioned_intervals = BTreeSet::default();
        positioned_intervals.insert(PositionedInterval::new(0, 64));

        let mut sized_intervals = BTreeSet::default();
        sized_intervals.insert(SizedInterval::new(0, 64));

        State {
            value: 0,
            positioned_intervals,
            sized_intervals,
        }
    }

    pub fn mask_on(&mut self, size: u8) -> Result<u64, String> {
        let err = String::from("could not assign interval");
        let parent = match self.sized_intervals.range(SizedInterval::new(0, size)..).next() {
            None => return Err(err),
            Some(parent) => parent.clone(),
        };
        let child = match self.sized_intervals.take(&parent) { None => return Err(err), Some(next) => next };
        self.positioned_intervals.insert(PositionedInterval::new(child.first() + size, parent.last()));
        self.sized_intervals.insert(SizedInterval::new(child.first() + size, parent.last()));
        Ok(2u64.checked_pow(size as u32).unwrap() - 1 << (64 - child.first() - size))
    }

    pub fn mask_off(&mut self, interval: Interval) {
        let previous = match self.positioned_intervals.range(..PositionedInterval(interval.clone())).next_back() {
            None => LEFT_IDENTIY_INTERVAL,
            Some(previous) => previous.0.clone(),
        };
        let next = match self.positioned_intervals.range(PositionedInterval(interval.clone())..).next_back() {
            None => RIGHT_IDENTIY_INTERVAL,
            Some(next) => next.0.clone(),
        };

        let mut parent = interval.clone();

        if previous.last() == interval.first() && previous.last() != 0 {
            self.positioned_intervals.take(&PositionedInterval(previous.clone()));
            self.sized_intervals.take(&SizedInterval(previous.clone()));
            parent.set_first(previous.first());
        }

        if next.first() == interval.last() && next.first() != 64 {
            self.positioned_intervals.take(&PositionedInterval(next.clone()));
            self.sized_intervals.take(&SizedInterval(next.clone()));
            parent.set_last(next.last());
        }

        self.positioned_intervals.insert(PositionedInterval(parent.clone()));
        self.sized_intervals.insert(SizedInterval(parent.clone()));
    }
}

impl<'a> Eq for Cell<'a> {}

impl<'a> PartialEq for Cell<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.proto_tile.size() != other.proto_tile.size() {
            return false
        }
        return izip!(self.proto_tile.points.iter(), other.proto_tile.points.iter()).all(|(self_point, other_point)| self_point == other_point)
    }
}

pub type Rule = fn(proto_tile: &ProtoTile, cell: &Cell) -> u64;

impl<'a> Simulation<'a> {
    // pub fn new(tiling: Tiling, allowed_states_entries: Vec<(ProtoTile, u8)>) -> Simulation<'a> {
    //     let allowed_states: HashMap<ProtoTile, u8> = HashMap::default();
    //     for (proto_tile, allowed_state) in allowed_states_entries.into_iter() {
    //         allowed_states.insert(proto_tile, allowed_state);
    //     }
    //     Simulation {
    //         tiling,
    //         allowed_states,
    //         cells: BTreeMap::default(),
    //         generation: 0,
    //         states: [Vec::new(), Vec::new()],
    //     }
    // }
    //
    // pub fn init(&self, cells: Vec<(ProtoTile, u8)>) {
    //     for (proto_tile, state) in cells.into_iter() {
    //         self.insert_cell(proto_tile, state);
    //     }
    // }
    //
    // pub fn step() {
    //
    // }
    //
    // fn insert_cell(&self, proto_tile: ProtoTile, state: u8) {
    //     proto_tile.reorient_about_origin();
    //     let cell = Cell {
    //         proto_tile,
    //     };
    //     cells.insert(self.generation, cell);
    // }
}
