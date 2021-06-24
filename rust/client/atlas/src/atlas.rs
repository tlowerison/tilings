use common::{DEFAULT_F64_MARGIN, fmt_float, rad};
use float_cmp::ApproxEq;
use geometry::{Euclid, Point, ORIGIN, Transformable};
use itertools::{Itertools, izip};
use models;
use std::{collections::HashSet, f64::consts::{PI, TAU}, iter};
use tile::ProtoTile;

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
    pub proto_neighbors: Vec<ProtoNeighbor>, // proto_neighbors[i].transform.translate == proto_edges[i].proto_tile.points[i+1]
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

pub struct Atlas {
    pub proto_tiles: Vec<ProtoTile>,
    pub proto_vertex_stars: Vec<ProtoVertexStar>,
}

impl Atlas {
    pub fn new(config: models::FullAtlas) -> Result<Atlas, String> {
        // collect all proto tiles belonging to all vertices prior to
        // building vertices to be able to reference other vertices
        // while building their neighbors
        let mut all_proto_tiles: Vec<Vec<ProtoTile>> = Vec::with_capacity(config.vertices.len());
        for (i, vertex) in config.vertices.iter().enumerate() {
            let mut proto_tiles: Vec<ProtoTile> = Vec::with_capacity(vertex.edges.len());
            let mut rotation = 0.;
            for (j, edge) in vertex.edges.iter().enumerate() {
                let base_proto_tile = config.polygons
                    .get(edge.polygon_index)
                    .map(ProtoTile::from)
                    .ok_or(String::from(format!(
                        "polygon {} is missing in vertex {} spec",
                        edge.polygon_index,
                        j,
                    )))?;

                let point = match base_proto_tile.points.get(edge.point_index) {
                    Some(point) => point,
                    None => return Err(String::from(format!(
                        "vertex {}, edge {} has missing point for index {} - ProtoTile == {}",
                        i,
                        j,
                        edge.point_index,
                        base_proto_tile,
                    ))),
                };

                let mut proto_tile = base_proto_tile.transform(&Euclid::Translate(point.neg().values()));

                let next_point = match proto_tile.points.get((edge.point_index + 1) % base_proto_tile.size()) {
                    Some(point) => point.clone(),
                    None => return Err(String::from(format!(
                        "vertex {}, edge {} has missing point for index {} - ProtoTile == {}",
                        i,
                        j,
                        (edge.point_index + 1) % base_proto_tile.size(),
                        base_proto_tile,
                    ))),
                };

                let angle = proto_tile.angle(edge.point_index);

                proto_tile = proto_tile.transform(&Euclid::Rotate(-(next_point.arg() - rotation)));
                proto_tile.reorient(&ORIGIN);

                proto_tiles.extend(vec![proto_tile]);
                rotation += angle;
            }
            if !rotation.approx_eq(TAU, DEFAULT_F64_MARGIN) {
                return Err(String::from(format!("vertex {} - prototiles don't fit together perfectly - expected 360° fill but received ~{}°\n{}", i, fmt_float(rotation * 360. / TAU, 2), vertex)))
            }
            all_proto_tiles.extend(iter::once(proto_tiles));
        }

        let mut proto_vertex_stars: Vec<ProtoVertexStar> = Vec::with_capacity(config.vertices.len());
        for (i, (vertex, proto_tiles)) in izip!(config.vertices.iter(), all_proto_tiles.iter()).enumerate() {
            let mut proto_neighbors: Vec<ProtoNeighbor> = Vec::with_capacity(vertex.edges.len());
            for (edge, proto_tile) in izip!(vertex.edges.iter(), proto_tiles.iter()) {
                let edge_point = proto_tile.points.get(1).unwrap();

                let neighbor_edge_point = all_proto_tiles
                    .get(edge.neighbor_index)
                    .ok_or(String::from("Invalid neighbor index in edge"))?
                    .get(edge.point_index)
                    .ok_or(String::from("Invalid point index in edge"))?
                    .points
                    .get(1)
                    .unwrap();

                proto_neighbors.extend(vec![ProtoNeighbor {
                    proto_vertex_star_index: edge.neighbor_index,
                    neighbor_index: edge.polygon_index,
                    transform: VertexStarTransform {
                        parity: edge.parity,
                        translate: edge_point.values(),
                        rotate: rad(edge_point.neg().arg() - neighbor_edge_point.arg()),
                    },
                    forward_tile_index: i,
                    reverse_tile_index: (i + proto_tiles.len() - 1) % proto_tiles.len(),
                }]);
            }
            proto_vertex_stars.extend(vec![ProtoVertexStar {
                index: i,
                proto_tiles: proto_tiles.clone(),
                proto_neighbors,
            }]);
        }

        let mut proto_tiles: HashSet<ProtoTile> = HashSet::default();
        for vertex_proto_tiles in all_proto_tiles.into_iter() {
            for proto_tile in vertex_proto_tiles.into_iter() {
                proto_tiles.insert(proto_tile);
            }
        }
        let mut proto_tiles = proto_tiles.into_iter().collect_vec();
        proto_tiles.shrink_to_fit();

        Ok(Atlas {
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
        match write!(f, "edges:\n") {
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

impl std::fmt::Display for Atlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = format!("Atlas");
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
