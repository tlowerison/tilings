use common::to_rad;
use geometry::{Euclid, ORIGIN, Point, Transformable};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tile::{ProtoTile, regular_polygon, star_polygon};

const X: Point = Point(1., 0.);

pub struct Component(
    pub ProtoTile, /* proto_tile */
    pub usize,     /* point_index */
);

impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Component({}, {})", self.0, self.1)
    }
}

pub struct Neighbor(
    pub usize, /* proto_vertex_star_index */
    pub usize, /* neighbor_index */
    pub bool,  /* parity */
);

impl std::fmt::Display for Neighbor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Neighbor({}, {}, {})", self.0, self.1, self.2)
    }
}

pub struct Vertex {
    pub components: Vec<Component>,
    pub neighbors: Vec<Neighbor>, // components[i], neighbors[i], components[i+1]
}

impl std::fmt::Display for Vertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vertex\n  components:\n{}  \n  neighbors:\n{}",
            self.components.iter().map(|component| format!("  - {}", component)).collect_vec().join("\n"),
            self.neighbors.iter().map(|neighbor| format!("  - {}", neighbor)).collect_vec().join("\n")
        )
    }
}

pub struct Tiling(pub Vec<Vertex>);

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SerializedProtoTile {
    #[serde(rename = "custom")]
    Custom { sides: Vec<(/* length */ f64, /* relative rotation */ f64)> },
    #[serde(rename = "regular")]
    Regular { side_length: f64, num_sides: usize },
    #[serde(rename = "star")]
    Star { side_length: f64, num_base_sides: usize, internal_angle: f64 },
}

impl SerializedProtoTile {
    pub fn as_proto_tile(&self) -> ProtoTile {
        match self {
            SerializedProtoTile::Custom { sides } => {
                let mut point = ORIGIN.clone();
                let mut rotation = 0.;

                let mut points: Vec<Point> = Vec::with_capacity(sides.len() + 1);
                points.extend(vec![point.clone()]);
                for (length, relative_rotation) in sides.iter() {
                    rotation += to_rad(180. - relative_rotation);
                    point = &point + &X.mul(*length).transform(&Euclid::Rotate(rotation));
                    points.extend(vec![point.clone()]);
                }
                ProtoTile::new(points)
            },
            SerializedProtoTile::Regular { side_length, num_sides } => regular_polygon(*side_length, *num_sides),
            SerializedProtoTile::Star { side_length, num_base_sides, internal_angle } => star_polygon(*side_length, *num_base_sides, to_rad(*internal_angle)),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct SerializedComponent {
    prototile: String,
    point: usize,
}

impl SerializedComponent {
    pub fn as_component(&self, proto_tiles: &HashMap<String, ProtoTile>) -> Result<Component, String> {
        let proto_tile = match proto_tiles.get(&self.prototile) {
            Some(proto_tile) => proto_tile,
            None => return Err(String::from(format!("no proto_tile found with name {}", &self.prototile))),
        };
        if self.point >= proto_tile.size() {
            return Err(String::from(format!("component was specified with invalid point for prototile {}: point out of bounds", &self.prototile)))
        }
        return Ok(Component(proto_tile.clone(), self.point))
    }

    pub fn update_name(&mut self, new_name: String) {
        self.prototile = new_name;
    }
}

#[derive(Deserialize, Serialize)]
pub struct SerializedNeighbor(Vec<usize>);

impl SerializedNeighbor {
    pub fn as_neighbor(&self) -> Result<Neighbor, String> {
        let len = self.0.len();
        if len < 2 || len > 3 {
            return Err(String::from(format!("neighbor expected 2-3 members, received [{}]", self.0.iter().map(|e| format!("{}", e)).join(","))))
        }
        let proto_vertex_star_index = *self.0.get(0).unwrap();
        let neighbor_index = *self.0.get(1).unwrap();
        if len == 2 {
            return Ok(Neighbor(proto_vertex_star_index, neighbor_index, false))
        }
        let parity = *self.0.get(2).unwrap();
        if !(parity == 0 || parity == 1) {
            return Err(String::from(format!("neighbor expected 3rd member to be 0 or 1, received [{}]", self.0.iter().map(|e| format!("{}", e)).join(","))))
        }
        return Ok(Neighbor(proto_vertex_star_index, neighbor_index, parity == 1))
    }
}

#[derive(Deserialize, Serialize)]
pub struct SerializedVertex(pub Vec<SerializedComponent>);

#[derive(Deserialize, Serialize)]
pub struct SerializedAdjacency {
    vertex: usize,
    neighbors: Vec<SerializedNeighbor>,
}

impl SerializedAdjacency {
    pub fn as_vertex(&self, proto_tiles: &HashMap<String, ProtoTile>, ser_vertices: &Vec<SerializedVertex>) -> Result<Vertex, String> {
        let ser_components = match ser_vertices.get(self.vertex) {
            Some(ser_vertex) => &ser_vertex.0,
            None => return Err(format!("out of bounds vertex index {}", self.vertex)),
        };
        let ser_neighbors = &self.neighbors;
        let mut components: Vec<Component> = Vec::with_capacity(ser_components.len());
        let mut neighbors: Vec<Neighbor> = Vec::with_capacity(ser_neighbors.len());

        for ser_component in ser_components.iter() {
            let component = match ser_component.as_component(&proto_tiles) {
                Ok(component) => component,
                Err(e) => return Err(e),
            };
            components.extend(vec![component]);
        }

        for ser_neighbor in ser_neighbors.iter() {
            let neighbor = match ser_neighbor.as_neighbor() {
                Ok(neighbor) => neighbor,
                Err(e) => return Err(e),
            };
            neighbors.extend(vec![neighbor]);
        }

        Ok(Vertex { components, neighbors })
    }
}

#[derive(Deserialize, Serialize)]
pub struct SerializedTiling {
    pub labels: Vec<String>,
    #[serde(rename = "prototiles")]
    pub proto_tiles: HashMap<String, SerializedProtoTile>,
    pub vertices: Vec<SerializedVertex>,
    pub adjacencies: Vec<SerializedAdjacency>
}

impl SerializedTiling {
    pub fn as_tiling(&self) -> Result<Tiling, String> {
        let proto_tiles: HashMap<String, ProtoTile> = self.proto_tiles.iter().map(|(name, ser_proto_tile)| (name.clone(), ser_proto_tile.as_proto_tile())).collect();
        let mut vertices: Vec<Vertex> = Vec::with_capacity(self.vertices.len());
        for ser_adjacency in self.adjacencies.iter() {
            let vertex = match ser_adjacency.as_vertex(&proto_tiles, &self.vertices) {
                Ok(vertex) => vertex,
                Err(e) => return Err(e),
            };
            vertices.extend(vec![vertex]);
        }
        Ok(Tiling(vertices))
    }

    pub fn obfuscate_proto_tile_names(mut self) -> SerializedTiling {
        let new_proto_tile_names: HashMap<String, String> = self.proto_tiles.keys().enumerate().map(|(i, name)| (name.clone(), SerializedTiling::obfuscated_name(i))).collect();
        self.labels.append(&mut self.proto_tiles.keys().map(|name| name.clone()).collect_vec());
        self.proto_tiles = self.proto_tiles.into_iter().map(|(name, proto_tile)| (new_proto_tile_names.get(&name).unwrap().clone(), proto_tile)).collect();
        self.vertices = self.vertices.into_iter().map(|vertex| SerializedVertex(
            vertex.0.into_iter().map(|mut ser_component| {
                let new_name = new_proto_tile_names.get(&ser_component.prototile).unwrap().clone();
                ser_component.update_name(new_name);
                ser_component
            }).collect()
        )).collect();
        self
    }

    fn obfuscated_name(index: usize) -> String {
        let len = ((index as f64).log(52.).max(0.) + 1.) as usize;
        let mut chars: Vec<u8> = Vec::with_capacity(len);
        let mut val = index;
        for _ in 0 .. len {
            let remainder = (val % 52) as u8;
            if remainder < 26 {
                chars.extend(vec![65 + remainder]);
            } else {
                chars.extend(vec![97 + remainder]);
            }
            val /= 52;
        }
        String::from_utf8(chars).unwrap()
    }
}

#[derive(Deserialize, Serialize)]
pub struct SerializedTilings(pub HashMap<String, SerializedTiling>);

impl SerializedTilings {
    pub fn obfuscate_proto_tile_names(self) -> SerializedTilings {
        SerializedTilings(
            self.0.into_iter().map(|(name, ser_tiling)| (name, ser_tiling.obfuscate_proto_tile_names())).collect()
        )
    }
}
