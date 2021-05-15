use crate::tile::{ProtoTile, Tile};
use common::{approx_eq, fmt_float};
use geometry::{Euclid, Point, Transformable, ORIGIN};
use itertools::{izip, Itertools};
use std::{collections::HashSet, f64::consts::TAU};

// abstract graph - tiling

// ProtoComponent represents a point in a prototile which
// is a member of a vertex star.
pub struct ProtoComponent {
    pub index: usize,
    pub proto_tile: ProtoTile,
    pub point_index: usize,
}

#[derive(Clone)]
pub struct ProtoNeighbor {
    pub proto_vertex_star_index: usize,
    pub transform: VertexStarTransform,
    pub neighbor_index: usize,
}

pub struct ProtoVertexStar {
    pub index: usize,
    pub proto_components: Vec<ProtoComponent>,
    pub proto_neighbors: Vec<ProtoNeighbor>, // proto_neighbors[i].transform.translate == proto_components[i].proto_tile.points[i+1]
}

#[derive(Clone)]
pub struct VertexStarTransform {
    pub flip: bool,
    pub translate: (f64, f64),
    pub rotate: f64,
}

impl ProtoVertexStar {
    pub fn size(&self) -> usize {
        self.proto_components.len()
    }

    pub fn index(&self, index: usize, delta: isize) -> usize {
        ProtoVertexStar::index_from_size(self.size(), index, delta)
    }

    pub fn index_from_size(size: usize, index: usize, delta: isize) -> usize {
        let size = size as isize;
        ((index as isize + (delta % size) + size) % size) as usize
    }
}

pub mod config {
    use super::ProtoTile;
    pub struct Component(
        pub ProtoTile, /* proto_tile */
        pub usize,     /* point_index */
    );

    pub struct Neighbor(
        pub usize, /* proto_vertex_star_index */
        pub usize, /* neighbor_index */
        pub bool,  /* flipped */
    );

    pub struct Vertex {
        pub components: Vec<Component>,
        pub neighbors: Vec<Neighbor>, // components[i], neighbors[i], components[i+1]
    }

    pub struct Config(pub Vec<Vertex>);
}

pub struct Tiling {
    pub name: String,
    pub tiles: Vec<Tile>,
    pub proto_vertex_stars: Vec<ProtoVertexStar>,
}

impl Tiling {
    pub fn new(name: String, config: config::Config) -> Tiling {
        let all_proto_tiles: Vec<Vec<ProtoTile>> = config
            .0
            .iter()
            .map(|vertex| {
                let mut proto_tiles: Vec<ProtoTile> = vec![];
                let mut rotation = 0.;
                for component in vertex.components.iter() {
                    let point = match component.0.points.get(component.1) {
                        Some(point) => point,
                        None => panic!(
                            "missing point for index {} in ProtoTile {}",
                            component.1, component.0
                        ),
                    };
                    let mut proto_tile = component
                        .0
                        .transform(&Euclid::Translate(point.neg().values()));
                    let next_point = proto_tile
                        .points
                        .get((component.1 + 1) % component.0.size())
                        .unwrap()
                        .clone();
                    proto_tile =
                        proto_tile.transform(&Euclid::Rotate(-(next_point.arg() - rotation)));
                    proto_tile.reorient(&ORIGIN);
                    let angle = proto_tile.angle(component.1);
                    proto_tiles.push(proto_tile);
                    rotation += angle;
                }
                approx_eq!(f64, TAU, rotation);
                proto_tiles
            })
            .collect();

        let mut proto_vertex_stars = izip!(config.0.iter(), all_proto_tiles.iter()).enumerate().map(|(i,(vertex, proto_tiles))| {
            let proto_components = {
                let mut proto_components: Vec<ProtoComponent> = izip!(vertex.components.iter(), proto_tiles.iter()).enumerate().map(|(j,(component, proto_tile))| ProtoComponent {
                    proto_tile: proto_tile.clone(),
                    index: j,
                    point_index: component.1,
                }).collect();
                proto_components.shrink_to_fit();
                proto_components
            };

            let proto_neighbors = {
                let mut proto_neighbors: Vec<ProtoNeighbor> = izip!(vertex.neighbors.iter(), proto_components.iter()).map(|(neighbor, proto_component)| -> ProtoNeighbor {
                    let edge_point = proto_component.proto_tile.points.get(1).unwrap();
                    let neighbor_edge_point = all_proto_tiles.get(neighbor.0).unwrap().get(neighbor.1).unwrap().points.get(1).unwrap();
                    if neighbor.0 >= config.0.len() {
                        panic!("tiling does not have vertex star {} but vertex star {} lists it as a neighbor", neighbor.0, i);
                    }
                    println!("{} {}", proto_component.proto_tile, all_proto_tiles.get(neighbor.0).unwrap().get(neighbor.1).unwrap());
                    println!("{} {}", edge_point.neg(), neighbor_edge_point);
                    println!();
                    let mut rotate = edge_point.neg().arg() - neighbor_edge_point.arg();
                    if (rotate.abs() % TAU) < 0.0001 {
                        rotate = 0.
                    }
                    ProtoNeighbor {
                        proto_vertex_star_index: neighbor.0,
                        neighbor_index: neighbor.1,
                        transform: VertexStarTransform {
                            flip: neighbor.2,
                            translate: edge_point.values(),
                            rotate,
                        }
                    }
                }).collect();
                proto_neighbors.shrink_to_fit();
                proto_neighbors
            };

            ProtoVertexStar {
                index: i,
                proto_components,
                proto_neighbors,
            }
        }).collect::<Vec<ProtoVertexStar>>();
        proto_vertex_stars.shrink_to_fit();

        let mut tiles: HashSet<Tile> = HashSet::default();
        for vertex in config.0.iter() {
            for component in vertex.components.iter() {
                tiles.insert(Tile::new(component.0.clone()));
            }
        }

        Tiling {
            name,
            proto_vertex_stars,
            tiles: tiles.into_iter().collect_vec(),
        }
    }
}

// Display

impl std::fmt::Display for ProtoComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "- {}", self.proto_tile)
    }
}

impl std::fmt::Display for ProtoNeighbor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.proto_vertex_star_index, self.neighbor_index, self.transform
        )
    }
}

impl std::fmt::Display for ProtoVertexStar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match write!(f, "{}-components:\n", self.index) {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        for proto_component in self.proto_components.iter() {
            match write!(f, "{}\n", proto_component) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }
        match write!(f, "\n") {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        match write!(f, "{}-neighbors:\n", self.index) {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        for proto_neighbor in self.proto_neighbors.iter() {
            match write!(f, "{}\n", proto_neighbor) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for Tiling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = format!("Tiling {}", self.name);
        match write!(f, "{}\n{}\n", title, "-".repeat(title.len())) {
            Ok(_) => {}
            Err(e) => return Err(e),
        };
        for proto_vertex_star in self.proto_vertex_stars.iter() {
            match write!(
                f,
                "adjacencies:\n{}: {}",
                proto_vertex_star.index,
                proto_vertex_star
                    .proto_neighbors
                    .iter()
                    .map(|proto_neighbor| format!(
                        "({},{})",
                        proto_neighbor.proto_vertex_star_index, proto_neighbor.neighbor_index
                    ))
                    .collect::<Vec<String>>()
                    .join(" ")
            ) {
                Ok(_) => {}
                Err(e) => return Err(e),
            };
            match write!(f, "\n") {
                Ok(_) => {}
                Err(e) => return Err(e),
            };
        }
        match write!(f, "\n") {
            Ok(_) => {}
            Err(e) => return Err(e),
        };
        for (i, proto_vertex_star) in self.proto_vertex_stars.iter().enumerate() {
            match write!(f, "{}", proto_vertex_star) {
                Ok(_) => {}
                Err(e) => return Err(e),
            };
            if i < self.proto_vertex_stars.len() - 1 {
                match write!(f, "\n") {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                };
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for VertexStarTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{flip: {}, translate: {}, rotate: {}}}",
            self.flip,
            Point::new(self.translate),
            fmt_float(self.rotate, 3)
        )
    }
}
