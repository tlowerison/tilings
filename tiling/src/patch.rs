use crate::tile::*;
use crate::tiling::*;

use common::*;
use geometry::*;
use itertools::*;
use std::collections::{HashMap, HashSet, VecDeque, hash_map::Entry};

#[derive(Clone)]
pub struct VertexStar {
    pub point: Point,
    pub proto_vertex_star_index: usize,
    pub flip: bool,
    pub x_axis: f64, // x_axis of vertex_star
    components: HashSet<usize>,
}

pub enum VertexStarErr {
    BadProtoRef,
    BadRef,
    ComponentOutOfBounds(usize, Point),
    ProtoVertexStarErr(String),
}

impl std::fmt::Display for VertexStarErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VertexStarErr::BadRef => write!(f, "VertexStarErr: bad reference"),
            VertexStarErr::BadProtoRef => write!(f, "VertexStarErr: bad proto reference"),
            VertexStarErr::ComponentOutOfBounds(component_index, vertex_star_point) => write!(f, "VertexStarErr: component {} out of bounds in vertex star {}", component_index, vertex_star_point),
            VertexStarErr::ProtoVertexStarErr(value) => write!(f, "VertexStarErr: {}", value),
        }
    }
}

impl VertexStar {
    pub fn new(point: Point, proto_vertex_star_index: usize, flip: bool, x_axis: f64) -> VertexStar {
        VertexStar {
            point,
            proto_vertex_star_index,
            flip,
            x_axis,
            components: HashSet::default(),
        }
    }

    pub fn capacity<'a>(&self, tiling: &'a Tiling) -> Option<usize> {
        match self.get_proto_vertex_star(tiling) {
            None => None,
            Some(proto_vertex_star) => Some(proto_vertex_star.size()),
        }
    }

    pub fn get_index_with_flip(&self, tiling: &Tiling, index: usize) -> Option<usize> {
        let capacity = match self.capacity(tiling) { None => return None, Some(capacity) => capacity };
        Some(if self.flip { capacity - index - 1 } else { index })
    }

    pub fn get_proto_tile(&self, tiling: &Tiling, component_index: usize) -> Option<ProtoTile> {
        let proto_component = match self.get_proto_component(tiling, component_index) {
            None => return None,
            Some(proto_component) => proto_component,
        };
        let mut proto_tile = proto_component.proto_tile.transform(&Euclid::Rotate(self.x_axis));
        if self.flip {
            proto_tile = proto_tile.transform(&Euclid::Flip(self.x_axis));
        }
        Some(proto_tile.transform(&Euclid::Translate((self.point.0, self.point.1))))
    }

    pub fn get_link(&self, tiling: &Tiling) -> Option<Vec<Point>> {
        let proto_vertex_star = match self.get_proto_vertex_star(tiling) {
            None => return None,
            Some(proto_vertex_star) => proto_vertex_star,
        };
        let proto_neighbors = (0..proto_vertex_star.size()).filter_map(|neighbor_index| self.get_proto_neighbor(tiling, neighbor_index)).collect::<Vec<&ProtoNeighbor>>();
        if proto_neighbors.len() != proto_vertex_star.size() {
            return None
        }
        Some(proto_neighbors.iter().map(|proto_neighbor| &self.point + &Point::new(proto_neighbor.transform.translate)).collect())
    }

    pub fn get_component_edges(&self, tiling: &Tiling, component_index: usize) -> Option<Vec<(Point, Point)>> {
        let mut proto_tile = match self.get_proto_tile(tiling, component_index) {
            None => return None,
            Some(proto_tile) => proto_tile,
        };
        proto_tile.reorient(&self.point);
        let mut proto_tile_shifted_points = proto_tile.points.clone().into_iter().collect::<VecDeque<Point>>();
        proto_tile_shifted_points.rotate_left(1);
        Some(izip!(proto_tile.points.into_iter(), proto_tile_shifted_points.into_iter()).collect_vec())
    }

    pub fn get_proto_component<'a>(&self, tiling: &'a Tiling, component_index: usize) -> Option<&'a ProtoComponent> {
        let index = match self.get_index_with_flip(tiling, component_index) { None => return None, Some(index) => index };
        let proto_vertex_star = match self.get_proto_vertex_star(tiling) { None => return None, Some(proto_vertex_star) => proto_vertex_star };
        proto_vertex_star.proto_components.get(index)
    }

    pub fn get_proto_neighbor<'a>(&self, tiling: &'a Tiling, neighbor_index: usize) -> Option<&'a ProtoNeighbor> {
        let index = match self.get_index_with_flip(tiling, neighbor_index) { None => return None, Some(index) => index };
        let proto_vertex_star = match self.get_proto_vertex_star(tiling) {
            None => return None,
            Some(proto_vertex_star) => proto_vertex_star,
        };
        proto_vertex_star.proto_neighbors.get(index)
    }

    pub fn get_proto_vertex_star<'a>(&self, tiling: &'a Tiling) -> Option<&'a ProtoVertexStar> {
        tiling.proto_vertex_stars.get(self.proto_vertex_star_index)
    }

    pub fn has_component(&self, tiling: &Tiling, index: usize) -> bool {
        let index = match self.get_index_with_flip(tiling, index) { None => return false, Some(index) => index };
        match self.components.get(&index) { None => false, Some(_) => true }
    }

    pub fn size(&self) -> usize {
        self.components.len()
    }
}

impl std::fmt::Display for VertexStar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VertexStar:\n- point: {}\n- proto_vertex_star_index: {}\n- flip: {}\n- x_axis: {}",
            self.point,
            self.proto_vertex_star_index,
            self.flip,
            fmt_float(self.x_axis, 3),
        )
    }
}

pub enum TileDiff {
    Added,
    Removed,
}

pub struct Patch {
    pub tiling: Tiling,
    pub tiles: HashMap<Point, Tile>,
    pub tile_diffs: HashMap<Tile, TileDiff>,
    pub vertex_stars: HashMap<Point, VertexStar>,
}

// Path represents a path through a tiling, multiple of which
// can be used to construct a Patch.
pub struct Path {
    pub vertex_star_point: Point,
    pub component_index: usize,
    pub edge_indices: Vec<usize>,
}

struct InternalPath {
    pub vertex_star: VertexStar,
    pub component_index: usize,
    pub edge_indices: Vec<usize>,
}

pub enum PathErr {
    Missing(String),
    VertexStarErr(VertexStarErr),
}

impl std::fmt::Display for PathErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathErr::Missing(value) => write!(f, "PathErr: missing {}", value),
            PathErr::VertexStarErr(value) => write!(f, "PathErr: {}", value),
        }
    }
}

impl Patch {
    pub fn new(tiling: Tiling) -> Patch {
        let mut patch = Patch {
            tiling,
            tile_diffs: HashMap::default(),
            tiles: HashMap::default(),
            vertex_stars: HashMap::default(),
        };
        let point = Point(0.,0.);
        patch.vertex_stars.insert(point.clone(), VertexStar::new(point, 0, false, 0.));
        patch
    }

    // pub fn add_component(&mut self, centroid: &Point, edge: (Point, Point)) -> Result<Point, String> {
    //     let tile = match self.tiles.get(centroid) { Some(tile) => tile, None => return Err(String::from(format!("no component found with center at {}", centroid))) };
    //     let vertex_star_0 = match self.vertex_stars.get(&edge.0) { Some(vertex_star) => vertex_star, None => return Err(String::from(format!("no vertex found at {}", edge.0))) };
    //     let vertex_star_1 = match self.vertex_stars.get(&edge.1) { Some(vertex_star) => vertex_star, None => return Err(String::from(format!("no vertex found at {}", edge.0))) };
    //
    // }

    pub fn add_path(&mut self, mut path: Path) -> Result<(Point, usize), PathErr> {
        match self.vertex_stars.entry(path.vertex_star_point) {
            Entry::Occupied(vertex_star) => {
                path.edge_indices.reverse();
                let proto_vertex_star_index = vertex_star.get().proto_vertex_star_index;
                return self.add_path_component(InternalPath {
                    vertex_star: VertexStar::new(path.vertex_star_point, proto_vertex_star_index, false, 0.),
                    component_index: path.component_index,
                    edge_indices: path.edge_indices,
                })
            },
            Entry::Vacant(_) => return Err(PathErr::Missing(String::from("VertexStar"))),
        }
    }

    pub fn drain_tile_diffs(&mut self) -> HashMap<Tile, TileDiff> {
        self.tile_diffs.drain().collect()
    }

    pub fn drain_tiles(&mut self) -> HashMap<Tile, TileDiff> {
        self.tiles.drain().into_iter().map(|(_, tile)| (tile, TileDiff::Removed)).collect()
    }

    fn add_path_component(&mut self, mut path: InternalPath) -> Result<(Point, usize), PathErr> {
        match {
            self.vertex_stars.entry(path.vertex_star.point.clone()).or_insert(path.vertex_star.clone());
            let edge_index = match path.edge_indices.pop() { Some(edge_index) => edge_index, None => usize::MAX };
            match self.insert_component(&path.vertex_star.point, path.component_index, edge_index) {
                Ok(next) => next,
                Err(err) => return Err(PathErr::VertexStarErr(err)),
            }
        } {
            None => return Ok((path.vertex_star.point, path.component_index)),
            Some((vertex_star, component_index)) => {
                return self.add_path_component(
                    InternalPath {
                        vertex_star,
                        component_index,
                        edge_indices: path.edge_indices,
                    },
                )
            },
        }
    }

    fn insert_component(&mut self, point: &Point, component_index: usize, edge_index: usize) -> Result<Option<(VertexStar, usize)>, VertexStarErr> {
        let vertex_star = match self.vertex_stars.get_mut(point) { None => return Err(VertexStarErr::BadRef), Some(vertex_star) => vertex_star };
        let proto_vertex_star = match vertex_star.get_proto_vertex_star(&self.tiling) { None => return Err(VertexStarErr::BadProtoRef), Some(proto_vertex_star) => proto_vertex_star };
        let proto_component = match proto_vertex_star.proto_components.get(component_index) {
            None => return Err(VertexStarErr::ComponentOutOfBounds(component_index, point.clone())),
            Some(proto_component) => proto_component,
        };
        if edge_index >= proto_component.proto_tile.size() && edge_index != usize::MAX {
            return Err(VertexStarErr::ProtoVertexStarErr(format!(
                "edge_index {} is out of bounds for proto_tile {} in component {} of vertex_star {}",
                edge_index,
                proto_component.proto_tile,
                component_index,
                point,
            )));
        }

        match vertex_star.components.get(&component_index) {
            Some(_) => {},
            None => {
                match vertex_star.get_proto_tile(&self.tiling, component_index.clone()) {
                    None => return Err(VertexStarErr::BadRef),
                    Some(proto_tile) => {
                        let tile = Tile::new(proto_tile.clone());
                        self.tiles.entry(tile.centroid).or_insert({
                            self.tile_diffs.insert(tile.clone(), TileDiff::Added);
                            tile
                        });
                    },
                };
            },
        };
        vertex_star.components.insert(component_index);
        self.insert_component_link(point, component_index, edge_index)
    }

    fn insert_component_link(&mut self, init_point: &Point, init_component_index: usize, edge_index: usize) -> Result<Option<(VertexStar, usize)>, VertexStarErr> {
        let component_size = {
            let vertex_star = match self.vertex_stars.get(&init_point) {
                None => return Err(VertexStarErr::BadRef),
                Some(vertex_star) => vertex_star,
            };
            match vertex_star.get_proto_component(&self.tiling, init_component_index) {
                None => return Err(VertexStarErr::BadProtoRef),
                Some(proto_component) => proto_component.proto_tile.size(),
            }
        };

        let mut result: Option<(VertexStar, usize)> = if edge_index != 0 { None } else {
            let vertex_star = match self.vertex_stars.get(init_point) { None => return Err(VertexStarErr::BadRef), Some(vertex_star) => vertex_star.clone() };
            let proto_vertex_star = match vertex_star.get_proto_vertex_star(&self.tiling) { None => return Err(VertexStarErr::BadProtoRef), Some(proto_vertex_star) => proto_vertex_star };
            Some((
                vertex_star,
                proto_vertex_star.index(init_component_index, -1),
            ))
        };

        let mut component_index;
        let mut neighbor_index = init_component_index;
        let mut point = init_point.clone();
        let mut proto_neighbor;
        let mut vertex_star = match self.vertex_stars.get_mut(&point) { None => return Err(VertexStarErr::BadRef), Some(vertex_star) => vertex_star };

        for i in 1..component_size {
            let proto_vertex_star = match vertex_star.get_proto_vertex_star(&self.tiling) {
                None => return Err(VertexStarErr::BadProtoRef),
                Some(proto_vertex_star) => proto_vertex_star,
            };

            let next_proto_neighbor = match proto_vertex_star.proto_neighbors.get(neighbor_index) {
                Some(proto_neighbor) => proto_neighbor,
                None => return Err(VertexStarErr::ProtoVertexStarErr(format!(
                    "proto vertex star {} does not have neighbor {}",
                    proto_vertex_star.index,
                    neighbor_index,
                ))),
            };

            proto_neighbor = next_proto_neighbor.clone();
            neighbor_index = match vertex_star.get_index_with_flip(&self.tiling, proto_vertex_star.index(proto_neighbor.neighbor_index, -1)) {
                None => return Err(VertexStarErr::ProtoVertexStarErr(String::from("bad index flip"))),
                Some(neighbor_index) => neighbor_index,
            };
            component_index = proto_vertex_star.index(neighbor_index, -1);

            let x_axis = vertex_star.x_axis;

            let to_point = Patch::to_point(&point, &proto_neighbor);
            if let Err(e) = self.insert_vertex_star(x_axis, &to_point, &proto_neighbor) {
                return Err(e);
            }

            point = to_point;

            vertex_star = match self.vertex_stars.get_mut(&point) { None => return Err(VertexStarErr::BadRef), Some(vertex_star) => vertex_star };

            if i == edge_index {
                result = Some(
                    (VertexStar::new(
                        to_point.clone(),
                        proto_neighbor.proto_vertex_star_index,
                        proto_neighbor.transform.flip,
                        proto_neighbor.transform.rotate + x_axis,
                    ),
                    component_index,
                ));
            }
        }
        Ok(result)
    }

    fn insert_vertex_star(&mut self, x_axis: f64, point: &Point, to: &ProtoNeighbor) -> Result<(), VertexStarErr> {
        println!("{} {} {}", point, x_axis, to);
        self.vertex_stars.entry(point.clone()).or_insert(VertexStar::new(
            point.clone(),
            to.proto_vertex_star_index,
            to.transform.flip,
            to.transform.rotate + x_axis,
        ));

        let (proto_vertex_star_size, proto_neighbors) = match self.tiling.proto_vertex_stars.get(to.proto_vertex_star_index) {
            None => return Err(VertexStarErr::BadRef),
            Some(proto_vertex_star) => (proto_vertex_star.size(), proto_vertex_star.proto_neighbors.clone())
        };

        let components = match proto_neighbors.iter().enumerate()
            .filter_map(|(neighbor_index, proto_neighbor)| match self.vertex_stars.get(&point.transform(&Euclid::Translate(proto_neighbor.transform.translate))) {
                None => None,
                Some(neighbor_vertex_star) => match neighbor_vertex_star.get_proto_vertex_star(&self.tiling) {
                    None => None,
                    Some(neighbor_proto_vertex_star) => Some((neighbor_index, proto_neighbor, neighbor_vertex_star, neighbor_proto_vertex_star)),
                },
            })
            .map(|(neighbor_index, proto_neighbor, neighbor_vertex_star, neighbor_proto_vertex_star)| {
                let mut components: HashSet<usize> = HashSet::default();

                let neighbor_component_index = neighbor_proto_vertex_star.index(proto_neighbor.neighbor_index, 0);

                if neighbor_vertex_star.has_component(&self.tiling, neighbor_component_index) {
                    components.insert(ProtoVertexStar::index_from_size(proto_vertex_star_size, neighbor_index, -1));
                }

                let neighbor_component_index = neighbor_proto_vertex_star.index(proto_neighbor.neighbor_index, -1);

                if neighbor_vertex_star.has_component(&self.tiling, neighbor_component_index) {
                    components.insert(ProtoVertexStar::index_from_size(proto_vertex_star_size, neighbor_index, 0));
                }

                components
            })
            .reduce(|mut acc,e| { e.into_iter().for_each(|component| { acc.insert(component); }); acc })
        { Some(components) => components, None => return Ok(()) };

        let vertex_star = match self.vertex_stars.get_mut(point) { None => return Err(VertexStarErr::BadProtoRef), Some(vertex_star) => vertex_star };
        vertex_star.components = components;
        for component_index in vertex_star.components.iter() {
            match vertex_star.components.get(&component_index) {
                Some(_) => {},
                None => {
                    match vertex_star.get_proto_tile(&self.tiling, component_index.clone()) {
                        None => return Err(VertexStarErr::BadRef),
                        Some(proto_tile) => {
                            let tile = Tile::new(proto_tile.clone());
                            self.tiles.entry(tile.centroid).or_insert({
                                self.tile_diffs.insert(tile.clone(), TileDiff::Added);
                                tile
                            });
                        },
                    };
                },
            };
        }
        Ok(())
    }

    fn to_point(from: &Point, to: &ProtoNeighbor) -> Point {
        from + &Point::new(to.transform.translate)
    }
}

impl std::fmt::Display for Patch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match writeln!(f, "Patch:") { Ok(_) => {}, Err(e) => return Err(e) }
        for vertex_star in self.vertex_stars.values() {
            match writeln!(f, "{}", vertex_star) { Ok(_) => {}, Err(e) => return Err(e) }
            match writeln!(f, "- components:") { Ok(_) => {}, Err(e) => return Err(e) }
            for component in vertex_star.components.iter() {
                match writeln!(f, "  - {}", match vertex_star.get_proto_tile(&self.tiling, *component) {
                    None => return Err(std::fmt::Error),
                    Some(proto_tile) => proto_tile,
                }) { Ok(_) => {}, Err(e) => return Err(e) }
            }
        }
        match writeln!(f, "\n- tiles:") { Ok(_) => {}, Err(e) => return Err(e) }
        for (_, tile) in self.tiles.iter() {
            match writeln!(f, "  - {}", tile.proto_tile) { Ok(_) => {}, Err(e) => return Err(e) }
        }
        Ok(())
    }
}
