use geometry::Point;
use itertools::Itertools;
use serde::Deserialize;
use serde_json;
use tiling;

#[derive(Deserialize)]
pub struct Component(pub Vec<(f64, f64)>, pub usize, pub Option<Vec<(f64, f64)>>);

#[derive(Deserialize)]
pub struct Neighbor(pub usize, pub usize, pub bool);

#[derive(Deserialize)]
pub struct Vertex {
    pub components: Vec<Component>,
    pub neighbors: Vec<Neighbor>,
}

#[derive(Deserialize)]
pub struct Config(pub Vec<Vertex>);

pub fn deserialize(data: &str) -> Result<tiling::config::Config, String> {
    match serde_json::from_str::<Config>(data) {
        Ok(config) => Ok(tiling::config::Config(config.0.into_iter().map(|vertex| tiling::config::Vertex {
            components: vertex.components.into_iter().map(|component| tiling::config::Component(tiling::ProtoTile::new(component.0.into_iter().map(|values| Point::new(values)).collect_vec()), component.1)).collect_vec(),
            neighbors: vertex.neighbors.into_iter().map(|neighbor| tiling::config::Neighbor(neighbor.0, neighbor.1, neighbor.2)).collect_vec(),
        }).collect_vec())),
        Err(e) => Err(String::from(format!("error deserializing: {}", e))),
    }
}
