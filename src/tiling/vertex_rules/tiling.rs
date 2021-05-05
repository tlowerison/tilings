use core::fmt;
use float_cmp::*;
use itertools::*;
use crate::common::*;
use crate::tile::ProtoTile;

// abstract graph - tiling

// ProtoComponent represents a point in a prototile which
// is a member of a vertex star.
pub struct ProtoComponent {
    pub index: usize,
    pub proto_tile: ProtoTile,
    pub point_index: usize,
}

pub struct ProtoNeighbor {
    pub proto_vertex_star_index: usize,
    pub component_index: usize,
    pub reversed: bool,
}

impl Copy for ProtoNeighbor {}
impl Clone for ProtoNeighbor {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct ProtoVertexStar {
    pub index: usize,
    pub proto_components: Vec<ProtoComponent>,
    pub proto_neighbors: Vec<ProtoNeighbor>,
}

pub struct Tiling {
    pub name: String,
    pub vertices: Vec<ProtoVertexStar>,
}

pub struct Component(
    pub(crate) /* proto_tile */ ProtoTile,
    pub(crate) /* point_index */ usize,
);
pub struct Neighbor(
    pub(crate) /* proto_vertex_star_index */ usize,
    pub(crate) /* component_index */ usize,
    pub(crate) /* reversed */ bool,
);

// ProtoVertexStarConfig
pub struct Vertex {
    pub components: Vec<Component>,
    // components[i], neighbors[i], components[i+1]
    pub neighbors: Vec<Neighbor>,
}

pub struct Config(pub(crate) Vec<Vertex>);

impl Tiling {
    pub fn new(name: String, config: Config) -> Tiling {
        Tiling {
            name,
            vertices: config.0.into_iter().enumerate().map(|(i,proto_vertex_star_config)| {
                let mut rotation = 0.;
                let mut proto_tiles = vec![];
                for component_config in proto_vertex_star_config.components.iter() {
                    let proto_tile = &component_config.0;
                    let point_index = component_config.1;

                    let point = match proto_tile.0.get(point_index) { Some(point) => point, None => panic!("missing point for index {} in ProtoTile {}", point_index, proto_tile) };
                    let dp = &ORIGIN - point;

                    proto_tiles.push(proto_tile.transform(&Euclid::Translate(dp.0, dp.1).transform(&Euclid::Rotate(rotation))));
                    rotation += proto_tile.angle(point_index);
                }
                approx_eq!(f64, TAUD, rotation, ulps = 2);

                ProtoVertexStar {
                    index: i,
                    proto_components: izip!(proto_vertex_star_config.components.into_iter(), proto_tiles.into_iter()).enumerate().map(|(j,(component_config, proto_tile))| ProtoComponent {
                        proto_tile,
                        index: j,
                        point_index: component_config.1,
                    }).collect(),
                    proto_neighbors: proto_vertex_star_config.neighbors.into_iter().map(|neighbor_config| ProtoNeighbor {
                        proto_vertex_star_index: neighbor_config.0,
                        component_index: neighbor_config.1,
                        reversed: neighbor_config.2,
                    }).collect(),
                }
            }).collect::<Vec<ProtoVertexStar>>(),
        }
    }
}

// Display

impl fmt::Display for ProtoComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.point_index, self.proto_tile)
    }
}

impl fmt::Display for ProtoNeighbor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.proto_vertex_star_index, self.component_index, if self.reversed { "1" } else { "" })
    }
}

impl fmt::Display for ProtoVertexStar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = write!(f, "{}-components:\n", self.index); match result { Ok(_) => {}, Err(_) => return result };
        for (i, proto_component) in self.proto_components.iter().enumerate() {
            result = write!(f, "{}", proto_component); match result { Ok(_) => {}, Err(_) => return result };
            if i < self.proto_components.len() - 1 {
                result = write!(f, "\n"); match result { Ok(_) => {}, Err(_) => return result };
            }
        }
        result
    }
}

impl fmt::Display for Tiling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title = format!("Tiling {}", self.name);
        let mut result = write!(f, "{}\n{}\n", title, "-".repeat(title.len())); match result { Ok(_) => {}, Err(_) => return result };
        for proto_vertex_star in self.vertices.iter() {
            result = write!(f, "adjacencies:\n{}: {}", proto_vertex_star.index, proto_vertex_star.proto_neighbors.iter().map(|proto_neighbor| format!("({},{})", proto_neighbor.proto_vertex_star_index, proto_neighbor.component_index)).collect::<Vec<String>>().join(" ")); match result { Ok(_) => {}, Err(_) => return result };
            result = write!(f, "\n"); match result { Ok(_) => {}, Err(_) => return result };
        }
        result = write!(f, "\n"); match result { Ok(_) => {}, Err(_) => return result };
        for (i, proto_vertex_star) in self.vertices.iter().enumerate() {
            result = write!(f, "{}", proto_vertex_star); match result { Ok(_) => {}, Err(_) => return result };
            if i < self.vertices.len() - 1 {
                result = write!(f, "\n"); match result { Ok(_) => {}, Err(_) => return result };
            }
        }
        result
    }
}
