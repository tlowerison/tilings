use crate::tile::ProtoTile;
use common::{DEFAULT_F64_MARGIN, fmt_float, rad};
use float_cmp::ApproxEq;
use geometry::{Euclid, Point, Transformable, ORIGIN};
use itertools::{izip, Itertools};
use std::{collections::HashSet, f64::consts::{PI, TAU}, iter};

// abstract graph - tiling

#[derive(Clone)]
pub struct ProtoNeighbor {
    pub proto_vertex_star_index: usize,
    pub transform: VertexStarTransform,
    pub neighbor_index: usize,
    pub forward_tile_index: usize,
    pub reverse_tile_index: usize,
}

pub struct ProtoVertexStar {
    pub index: usize,
    pub proto_tiles: Vec<ProtoTile>,
    pub proto_neighbors: Vec<ProtoNeighbor>, // proto_neighbors[i].transform.translate == proto_components[i].proto_tile.points[i+1]
}

#[derive(Clone)]
pub struct VertexStarTransform {
    pub parity: bool,
    pub translate: (f64, f64),
    pub rotate: f64,
}

impl ProtoVertexStar {
    pub fn size(&self) -> usize {
        self.proto_tiles.len()
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
    use itertools::Itertools;

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

    pub struct Config(pub Vec<Vertex>);
}

pub struct Tiling {
    pub proto_tiles: Vec<ProtoTile>,
    pub proto_vertex_stars: Vec<ProtoVertexStar>,
}

impl Tiling {
    pub fn new(config: config::Config) -> Result<Tiling, String> {
        let mut all_proto_tiles: Vec<Vec<ProtoTile>> = Vec::with_capacity(config.0.len());
        for (i, vertex) in config.0.iter().enumerate() {
            let mut proto_tiles: Vec<ProtoTile> = Vec::with_capacity(vertex.components.len());
            let mut rotation = 0.;
            if vertex.components.len() != vertex.neighbors.len() {
                return Err(String::from(format!("vertex {} has mismatched # of components ({}) and neighbors ({})", i, vertex.components.len(), vertex.neighbors.len())))
            }
            for (j, component) in vertex.components.iter().enumerate() {
                if component.0.points.len() < 2 {
                    return Err(String::from(format!("vertex {}, component {} - expected >= 2 points but received {} points", i, j, component.0.points.len())))
                }

                let point = match component.0.points.get(component.1) {
                    Some(point) => point,
                    None => return Err(String::from(format!("vertex {}, component {} has missing point for index {} - ProtoTile == {}", i, j, component.1, component.0))),
                };

                let mut proto_tile = component.0.transform(&Euclid::Translate(point.neg().values()));

                let next_point_index = component.1;
                let next_point = match proto_tile.points.get((next_point_index + 1) % component.0.size()) {
                    Some(point) => point.clone(),
                    None => return Err(String::from(format!("vertex {}, component {} has missing point for index {} - ProtoTile == {}", i, j, (component.1 + 1) % component.0.size(), component.0))),
                };

                let angle = proto_tile.angle(next_point_index);

                proto_tile = proto_tile.transform(&Euclid::Rotate(-(next_point.arg() - rotation)));
                proto_tile.reorient(&ORIGIN);

                proto_tiles.extend_one(proto_tile);
                rotation += angle;
            }
            if !rotation.approx_eq(TAU, DEFAULT_F64_MARGIN) {
                return Err(String::from(format!("vertex {} - prototiles don't fit together perfectly - expected 360° fill but received ~{}°\n{}", i, fmt_float(rotation * 360. / TAU, 2), vertex)))
            }
            all_proto_tiles.extend(iter::once(proto_tiles));
        }

        let mut proto_vertex_stars: Vec<ProtoVertexStar> = Vec::with_capacity(config.0.len());
        for (i,(vertex, proto_tiles)) in izip!(config.0.iter(), all_proto_tiles.iter()).enumerate() {
            let mut proto_neighbors: Vec<ProtoNeighbor> = Vec::with_capacity(vertex.components.len());
            for (j, (neighbor, proto_tile)) in izip!(vertex.neighbors.iter(), proto_tiles.iter()).enumerate() {
                let edge_point = proto_tile.points.get(1).unwrap();
                let neighbor_edge_point = match all_proto_tiles.get(neighbor.0) {
                    None => return Err(String::from(format!("vertex {}, neighbor {} - no neighbor vertex found for index {}", i, j, neighbor.0))),
                    Some(neighbor_vs) => match neighbor_vs.get(neighbor.1) {
                        None => return Err(String::from(format!("vertex {}, neighbor {} - return edge {} is out of bounds for neighboring vertex star with size {}", i, j, neighbor.1, neighbor_vs.len()))),
                        Some(neighbor_proto_tile) => neighbor_proto_tile.points.get(1).unwrap(),
                    },
                };
                proto_neighbors.extend_one(ProtoNeighbor {
                    proto_vertex_star_index: neighbor.0,
                    neighbor_index: neighbor.1,
                    transform: VertexStarTransform {
                        parity: neighbor.2,
                        translate: edge_point.values(),
                        rotate: rad(edge_point.neg().arg() - neighbor_edge_point.arg()),
                    },
                    forward_tile_index: i,
                    reverse_tile_index: (i + proto_tiles.len() - 1) % proto_tiles.len(),
                });
            }
            proto_vertex_stars.extend_one(ProtoVertexStar {
                index: i,
                proto_tiles: proto_tiles.clone(),
                proto_neighbors,
            });
        }

        let mut proto_tiles: HashSet<ProtoTile> = HashSet::default();
        for vertex in config.0.iter() {
            for component in vertex.components.iter() {
                proto_tiles.insert(component.0.clone());
            }
        }
        let mut proto_tiles = proto_tiles.into_iter().collect_vec();
        proto_tiles.shrink_to_fit();

        Ok(Tiling {
            proto_vertex_stars,
            proto_tiles,
        })
    }
}

// Display

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
        match write!(f, "components:\n") {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        for proto_tile in self.proto_tiles.iter() {
            match write!(f, "{}\n", proto_tile) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }
        match write!(f, "\n") {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        match write!(f, "neighbors:\n") {
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
        let title = format!("Tiling");
        match write!(f, "{}\n{}\n", title, "-".repeat(title.len())) {
            Ok(_) => {}
            Err(e) => return Err(e),
        };
        for (i, proto_vertex_star) in self.proto_vertex_stars.iter().enumerate() {
            match write!(
                f,
                "adjacencies:\n{}: {}",
                i,
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
        match write!(f, "\nvertex stars:\n") {
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
            "{{parity: {}, translate: {}, rotate: {}π}}",
            self.parity,
            Point::new(self.translate),
            fmt_float(self.rotate / PI, 2)
        )
    }
}
