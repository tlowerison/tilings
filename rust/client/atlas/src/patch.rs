use crate::atlas::{Atlas, ProtoNeighbor, ProtoVertexStar};
use common::*;
use geometry::{Affine, Bounds, Euclid, Point, Spatial, Transform, Transformable};
use itertools::izip;
use pmr_quad_tree::{Config as TreeConfig, Tree};
use std::{
    borrow::Borrow,
    collections::{HashMap, VecDeque},
    iter,
    f64::consts::TAU,
};
use tile::Tile;

#[derive(Clone, Debug)]
pub struct VertexStar {
    pub proto_vertex_star_index: usize,
    pub point: Point,
    pub parity: bool, // whether the VertexStar's link is the cyclical reverse or not of the underlying ProtoVertexStar's link
    pub rotation: f64, // argument of VertexStar's first neighbor relative to the VertexStar's point
    pub link_vec: Vec<Point>, // points of VertexStar's neighboring VertexStars
    pub link_map: HashMap<Point, usize>, // each Point in the link maps to its index in link_vec
    pub link_args: Vec<f64>, // arg of each link Point relative to the central point
    pub link_arg_offset: usize,
}

#[derive(Debug)]
pub enum VertexStarErr {
    BadProtoRef,
    BadRef,
    ComponentOutOfBounds(usize, Point),
    ProtoVertexStarErr(String),
}

impl VertexStar {
    pub fn new(atlas: &Atlas, point: Point, proto_vertex_star_index: usize, parity: bool, rotation: f64) -> VertexStar {
        let proto_vertex_star = atlas.proto_vertex_stars.get(proto_vertex_star_index).unwrap();

        let mut link_vec: Vec<Point> = Vec::with_capacity(proto_vertex_star.size());

        let reference_frame = VertexStar::reference_frame(parity, rotation).transform(&Euclid::Translate(point.values()));

        let transform_link_point = |proto_neighbor: &ProtoNeighbor| Point::new(proto_neighbor.transform.translate).transform(&reference_frame);
        link_vec.extend(proto_vertex_star.proto_neighbors.iter().map(transform_link_point));

        let mut link_map: HashMap<Point, usize> = HashMap::with_capacity(proto_vertex_star.size());
        link_map.extend(link_vec.iter().enumerate().map(|(i, point)| (point.clone(), i)));

        let mut link_args_deque = link_vec.iter().map(|link_point| (link_point - &point).arg()).collect::<VecDeque<f64>>();
        println!("{:?}", link_args_deque.iter().map(|la| fmt_float::<f64>(*la, 2)).collect::<Vec<String>>());
        let mut link_arg_offset = 0;
        for (i, (arg_0, arg_1)) in izip!(link_args_deque.iter().take(link_args_deque.len() - 1), link_args_deque.iter().skip(1)).enumerate() {
            if arg_1 < arg_0 {
                link_args_deque.rotate_left(i + 1);
                link_arg_offset = i + 1;
                break
            }
        }

        let wrapped_left_arg = link_args_deque.get(link_vec.len() - 1).unwrap() - TAU;
        let wrapped_right_arg = link_args_deque.get(0).unwrap() + TAU;

        let mut link_args: Vec<f64> = Vec::with_capacity(proto_vertex_star.size() + 2);
        link_args.extend(
            iter::once(wrapped_left_arg)
                .chain(link_args_deque.into_iter())
                .chain(iter::once(wrapped_right_arg))
        );

        VertexStar {
            point,
            proto_vertex_star_index,
            parity,
            rotation,
            link_vec,
            link_map,
            link_args,
            link_arg_offset,
        }
    }

    pub fn reference_frame(parity: bool, rotation: f64) -> Affine {
        if !parity {
            return Euclid::Rotate(rotation).as_affine()
        }
        Euclid::Flip(0.).transform(&Euclid::Rotate(rotation)).as_affine()
    }

    // get_clockwise_adjacent_link_index optionally returns the cyclically preceding Point of the given Point
    // in this VertexStar's link (in counterclockwise cyclical ordering).
    pub fn get_clockwise_adjacent_link_index(&self, point: &Point) -> Option<usize> {
        let link_index = match self.link_map.get(point) { None => return None, Some(i) => *i };
        if !self.parity {
            Some((link_index + self.size() - 1) % self.size())
        } else {
            Some((link_index + self.size() - 2) % self.size()) // have had trouble with this line in the past, need to test against a vertex star with more than 3 components
        }
    }

    // get_neighbor_vertex_star optionally returns a VertexStar placed where this VertexStar's neighbor[index] specifies,
    // relative to this VertexStar's point, rotation and parity.
    pub fn get_neighbor_vertex_star(&self, atlas: &Atlas, index: usize) -> Option<VertexStar> {
        let proto_vertex_star = self.get_proto_vertex_star(atlas).unwrap();
        let proto_neighbor = proto_vertex_star.proto_neighbors.get(index).unwrap();
        let neighbor_proto_vertex_star = atlas.proto_vertex_stars.get(proto_neighbor.proto_vertex_star_index).unwrap();

        let reference_frame = VertexStar::reference_frame(self.parity, self.rotation);
        let neighbor_point_in_self_ref = Point::new(proto_neighbor.transform.translate).transform(&reference_frame);

        let neighbor_edge_point_in_neighbor_ref = Point::new(neighbor_proto_vertex_star.proto_neighbors.get(proto_neighbor.neighbor_index).unwrap().transform.translate);

        let neighbor_edge_rotation = rad((-neighbor_point_in_self_ref).arg() - neighbor_edge_point_in_neighbor_ref.arg());

        let mut neighbor_first_edge_point_transformed = Point::new(neighbor_proto_vertex_star.proto_neighbors.get(0).unwrap().transform.translate).transform(&Euclid::Rotate(neighbor_edge_rotation));

        let parity = self.mutual_parity(proto_neighbor.transform.parity);
        if parity {
            neighbor_first_edge_point_transformed = neighbor_first_edge_point_transformed.transform(&Euclid::Flip((-neighbor_point_in_self_ref).arg()));
        }

        let rotation = neighbor_first_edge_point_transformed.arg();

        Some(VertexStar::new(
            atlas,
            &self.point + &neighbor_point_in_self_ref,
            proto_neighbor.proto_vertex_star_index,
            parity,
            rotation,
        ))
    }

    // get_proto_vertex_star optionally returns this VertexStar's referenced ProtoVertexStar given a atlas to index into.
    pub fn get_proto_vertex_star<'a>(&self, atlas: &'a Atlas) -> Option<&'a ProtoVertexStar> {
        atlas.proto_vertex_stars.get(self.proto_vertex_star_index)
    }

    // get_tile creates the Tile situated clockwise of the given point in this VertexStar's link
    pub fn get_tile(&self, atlas: &Atlas, neighbor_point: &Point) -> Option<Tile> {
        let proto_vertex_star = match self.get_proto_vertex_star(atlas) { None => return None, Some(pvs) => pvs };
        let mut proto_tile_index = match self.link_map.get(neighbor_point) { None => return None, Some(i) => *i };
        if !self.parity {
            proto_tile_index = (proto_tile_index + self.size() - 1) % self.size();
        }
        let proto_tile = match proto_vertex_star.proto_tiles.get(proto_tile_index) { None => return None, Some(pt) => pt };
        let reference_frame = VertexStar::reference_frame(self.parity, self.rotation);
        Some(Tile::new(proto_tile.transform(&reference_frame.transform(&Euclid::Translate(self.point.values()))), self.parity))
    }

    // mutual_parity returns the XOR value of this VertexStar's parity with the provided parity.
    // This is useful for computing a new, neighboring VertexStar's parity.
    pub fn mutual_parity(&self, parity: bool) -> bool {
        self.parity ^ parity
    }

    // nearest_neighbor returns the neighbor_vertex_star closest to the provided point
    // as well as the point in this vertex star which is counter-clockwise of that point
    pub fn nearest_neighbor<'a>(&'a self, atlas: &Atlas, point: &Point) -> Option<VertexStar> {
        let arg = (point - &self.point).arg();
        let link_vec_index = (
            match self.link_args.binary_search_by(|link_arg| link_arg.partial_cmp(&arg).unwrap()) { Ok(i) => i, Err(i) => i }
            + self.size()
            + self.link_arg_offset
            - 1
        ) % self.size();

        println!("{} {:?} : {} -> {}", fmt_float::<f64>(arg, 2), self.link_args.iter().map(|la| fmt_float::<f64>(*la, 2)).collect::<Vec<String>>(), match self.link_args.binary_search_by(|link_arg| link_arg.partial_cmp(&arg).unwrap()) { Ok(i) => i, Err(i) => i }, link_vec_index);
        // println!("{:?}", self.link_vec);
        // let closest_two = (self.link_args.get(link_vec_insert_index).unwrap(), self.link_args.get(link_vec_insert_index + 1).unwrap());

        // let index = if arg - closest_two.0 < closest_two.1 - arg { index - 1 } else { index };
        // let mut index = (index + self.link_arg_offset + self.size() - 1) % self.size();

        // if Point::angle(self.link_vec.get(index).unwrap(), &self.point, point) >= PI {
        //     index = (index + 1 + self.link_arg_offset + self.size() - 1) % self.size();
        // }

        // println!(
        //     "{} {} {}",
        //     self.link_vec.get((index + self.size() - 1) % self.size()).unwrap(),
        //     self.link_vec.get((index + self.size() + 0) % self.size()).unwrap(),
        //     self.link_vec.get((index + self.size() + 1) % self.size()).unwrap(),
        // );

        self.get_neighbor_vertex_star(atlas, link_vec_index)
    }

    pub fn size(&self) -> usize {
        self.link_map.len()
    }
}

impl Spatial for VertexStar {
    type Hashed = Point;
    fn distance(&self, point: &Point) -> f64 { self.point.distance(point) }
    fn intersects(&self, bounds: &Bounds) -> bool { self.point.intersects(bounds) }
    fn key(&self) -> Self::Hashed { self.point.key() }
}

#[derive(Debug)]
pub enum TileDiff {
    Added,
    Removed,
}

#[derive(Debug)]
pub struct PatchTile {
    pub tile: Tile,
    pub included: bool,
}

impl Spatial for PatchTile {
    type Hashed = Point;
    fn distance(&self, point: &Point) -> f64 { self.tile.distance(point) }
    fn intersects(&self, bounds: &Bounds) -> bool { self.tile.intersects(bounds) }
    fn key(&self) -> Self::Hashed { self.tile.key() }
}

#[derive(Debug)]
pub struct Patch {
    pub atlas: Atlas,
    pub tile_diffs: HashMap<Tile, TileDiff>,
    pub vertex_stars: Tree<Point, VertexStar>,
    pub patch_tiles: Tree<Point, PatchTile>,
}

impl Patch {
    // new creates a new Patch and inserts a single VertexStar and its first Tile
    pub fn new(atlas: Atlas, tile_tree_config: TreeConfig, vertex_star_tree_config: TreeConfig) -> Result<Patch, String> {
        let mut vertex_stars: Tree<Point, VertexStar> = Tree::new(vertex_star_tree_config);
        vertex_stars.insert(VertexStar::new(&atlas, Point(0., 0.), 0, false, 0.));
        Ok(Patch {
            atlas,
            vertex_stars,
            tile_diffs: HashMap::default(),
            patch_tiles: Tree::new(tile_tree_config),
        })
    }

    pub fn drain_tile_diffs(&mut self) -> HashMap<Tile, TileDiff> {
        self.tile_diffs
            .drain()
            .collect()
    }

    pub fn insert_tile_by_point(&mut self, point: Point) -> Result<(), String> {
        let mut nearest_vertex_star_neighbor = self.vertex_stars.nearest_neighbor(&point).map_err(|e| format!("no nearby vertex stars:\n{}\n{:?}\n{:#?}", e, point, self.vertex_stars))?;
        let mut nearest_vertex_star_neighbor_rc = nearest_vertex_star_neighbor.item.upgrade();
        let mut nearest_vertex_star_rc = nearest_vertex_star_neighbor_rc.ok_or("vertex star doesn't exist")?;
        let mut nearest_vertex_star = nearest_vertex_star_rc.as_ref().borrow();

        let mut count = 0;
        loop {
            if count == 100 {
                return Err(format!("unable to add tile - too far"));
            }
            let next_vertex_star = nearest_vertex_star.nearest_neighbor(&self.atlas, &point).ok_or("failed to find nearest vertex star")?;

            let tile = nearest_vertex_star.get_tile(&self.atlas, &next_vertex_star.point).ok_or("couldn't get new tile")?;

            let edge = (nearest_vertex_star.point.clone(), next_vertex_star.point.clone());

            if tile.contains(&point) {
                self.insert_adjacent_tile_by_edge(edge, true)?;
                return Ok(())
            } else {
                self.insert_adjacent_tile_by_edge(edge, false)?;
                self.vertex_stars.insert(next_vertex_star);
                nearest_vertex_star_neighbor = self.vertex_stars.nearest_neighbor(&point).map_err(|e| format!("no nearby vertex stars:\n{}\n{:?}\n{:#?}", e, point, self.vertex_stars))?;
                nearest_vertex_star_neighbor_rc = nearest_vertex_star_neighbor.item.upgrade();
                nearest_vertex_star_rc = nearest_vertex_star_neighbor_rc.ok_or("vertex star doesn't exist")?;
                nearest_vertex_star = nearest_vertex_star_rc.as_ref().borrow();
            }
            count += 1;
        }
    }

    // insert_adjacent_tile_by_edge inserts a new Tile into this Patch
    // given a particular edge along which the Tile shares. In order to succeed,
    // both points in the edge are expected to be points of existing VertexStars
    // in this Patch. If both exist, the new Tile will be added starboard of the
    // edge drawn from start to stop.
    fn insert_adjacent_tile_by_edge(&mut self, (start, stop): (Point, Point), included: bool) -> Result<(), String> {
        let start_vertex_star = match self.vertex_stars.get(&start) { Some(vs) => vs, None => return Err(String::from(format!("no VertexStar found at start {}\n{:?}", start, self))) };
        let tile = match start_vertex_star.get_tile(&self.atlas, &stop) { Some(t) => t, None => return Err(String::from(format!("stop {} is not in the link of start {}\n{:?}", stop, start, self))) };
        let tile_size = tile.size();

        match self.patch_tiles.insert(PatchTile { tile: tile.clone(), included }) {
            None => {
                if included {
                    self.tile_diffs.insert(tile.clone(), TileDiff::Added);
                }
            },
            Some(patch_tile) => {
                if included && !patch_tile.included {
                    self.tile_diffs.insert(tile.clone(), TileDiff::Added);
                }
                return Ok(())
            },
        };

        let mut link_points: Vec<(usize, Point)> = vec![(0, stop.clone()), (0, start.clone())];
        let mut new_link_points: Vec<(usize, Point)> = vec![];
        let mut reverse = stop.clone();
        let mut middle = start.clone();

        for _ in 0 .. tile_size - 1 {
            let middle_vertex_star = match self.vertex_stars.get(&middle) { Some(vs) => vs, None => return Err(String::from(format!("missing VertexStar at {}\n{:?}", middle, self))) };
            let forward_index = match middle_vertex_star.get_clockwise_adjacent_link_index(&reverse) { Some(i) => i, None => return Err(String::from(format!("no link point found clockwise adjacent of {} for VertexStar {}\n{:?}", reverse, middle, self))) };
            let forward = match middle_vertex_star.link_vec.get(forward_index) { Some(p) => p.clone(), None => return Err(String::from(format!("out of bounds index {} in VertexStar {}\n{:?}", forward_index, middle, self))) };
            link_points.push((forward_index, forward));
            if let Some(vs) = self.vertex_stars.get(&forward) {
                reverse = middle;
                middle = vs.point.clone();
            } else {
                let vs = match middle_vertex_star.get_neighbor_vertex_star(&self.atlas, forward_index) { Some(vs) => vs, None => return Err(String::from(format!("unable to create neighbor VertexStar of VertexStar {} for neighbor index {} at point {}\n{:?}", middle, forward_index, forward, self))) };
                reverse = middle;
                middle = vs.point.clone();
                if !self.vertex_stars.has(&forward) {
                    new_link_points.push((forward_index, forward));
                    self.vertex_stars.insert(vs);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use tile::ProtoTile;
    use geometry::Point;
    use std::f64::consts::{PI, TAU};

    const ORIGIN: Point = Point(0., 0.);
    const X: Point = Point(1., 0.);
    const Y: Point = Point(0., 1.);

    fn get_tile_tree_config() -> TreeConfig {
        TreeConfig {
            initial_radius: 1000.,
            max_depth: 50,
            splitting_threshold: 25,
        }
    }

    fn get_vertex_star_tree_config() -> TreeConfig {
        TreeConfig {
            initial_radius: 1000.,
            max_depth: 70,
            splitting_threshold: 10,
        }
    }

    fn get_test_atlas_3_3_3_3_3_3() -> Atlas {
        Atlas {
            proto_tiles: vec![
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            1.0,
                            0.0,
                        ),
                        Point(
                            0.5000000000000002,
                            0.8660254037844388,
                        ),
                    ],
                    parity: false,
                },
            ],
            proto_vertex_stars: vec![
                ProtoVertexStar {
                    index: 0,
                    proto_tiles: vec![
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    1.0,
                                    0.0,
                                ),
                                Point(
                                    0.5000000000000002,
                                    0.8660254037844388,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    0.5000000000000003,
                                    0.8660254037844385,
                                ),
                                Point(
                                    -0.4999999999999997,
                                    0.866025403784439,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    -0.49999999999999944,
                                    0.866025403784439,
                                ),
                                Point(
                                    -1.0000000000000002,
                                    0.0000000000000007771561172376096,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    -1.0,
                                    0.0000000000000010106430996148606,
                                ),
                                Point(
                                    -0.5000000000000011,
                                    -0.8660254037844383,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    -0.5000000000000012,
                                    -0.8660254037844379,
                                ),
                                Point(
                                    0.49999999999999883,
                                    -0.8660254037844396,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    0.49999999999999856,
                                    -0.8660254037844395,
                                ),
                                Point(
                                    1.0000000000000002,
                                    -0.0000000000000017763568394002505,
                                ),
                            ],
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    1.0,
                                    0.0,
                                ),
                                rotate: 3.141592653589793,
                            },
                            neighbor_index: 3,
                            forward_tile_index: 0,
                            reverse_tile_index: 5,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    0.5000000000000003,
                                    0.8660254037844385,
                                ),
                                rotate: 4.18879020478639,
                            },
                            neighbor_index: 4,
                            forward_tile_index: 0,
                            reverse_tile_index: 5,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    -0.49999999999999944,
                                    0.866025403784439,
                                ),
                                rotate: 5.235987755982988,
                            },
                            neighbor_index: 5,
                            forward_tile_index: 0,
                            reverse_tile_index: 5,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    -1.0,
                                    0.0000000000000010106430996148606,
                                ),
                                rotate: 0.0,
                            },
                            neighbor_index: 0,
                            forward_tile_index: 0,
                            reverse_tile_index: 5,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    -0.5000000000000012,
                                    -0.8660254037844379,
                                ),
                                rotate: 1.0471975511965965,
                            },
                            neighbor_index: 1,
                            forward_tile_index: 0,
                            reverse_tile_index: 5,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    0.49999999999999856,
                                    -0.8660254037844395,
                                ),
                                rotate: 2.094395102393193,
                            },
                            neighbor_index: 2,
                            forward_tile_index: 0,
                            reverse_tile_index: 5,
                        },
                    ],
                },
            ],
        }
    }

    fn get_test_atlas_4_4_4_4() -> Atlas {
        Atlas {
            proto_tiles: vec![
                ProtoTile {
                    points: vec![
                        Point(0., 0.),
                        Point(1., 0.),
                        Point(1., 1.),
                        Point(0., 1.),
                    ],
                    parity: false,
                },
            ],
            proto_vertex_stars: vec![
                ProtoVertexStar {
                    index: 0,
                    proto_tiles: vec![
                        ProtoTile {
                            points: vec![
                                Point(0., 0.),
                                Point(1., 0.),
                                Point(1., 1.),
                                Point(0., 1.),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(0., 0.),
                                Point(0., 1.),
                                Point(-1., 1.),
                                Point(-1., 0.),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(0., 0.),
                                Point(-1., 0.),
                                Point(-1., -1.),
                                Point(-0., -1.),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(0., 0.),
                                Point(0., -1.),
                                Point(1., -1.),
                                Point(1., 0.),
                            ],
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    1.0,
                                    0.0,
                                ),
                                rotate: PI,
                            },
                            neighbor_index: 2,
                            forward_tile_index: 0,
                            reverse_tile_index: 3,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (0., 1.),
                                rotate: 3. * PI / 2.,
                            },
                            neighbor_index: 3,
                            forward_tile_index: 0,
                            reverse_tile_index: 3,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (-1., 0.),
                                rotate: 0.0,
                            },
                            neighbor_index: 0,
                            forward_tile_index: 0,
                            reverse_tile_index: 3,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (0., -1.),
                                rotate: PI / 2.,
                            },
                            neighbor_index: 1,
                            forward_tile_index: 0,
                            reverse_tile_index: 3,
                        },
                    ],
                },
            ],
        }
    }

    fn get_test_atlas_6_6_6() -> Atlas {
        Atlas {
            proto_tiles: vec![
                ProtoTile {
                    points: vec![
                        Point(0., 0.),
                        Point(1., 0.),
                        Point(1.5, 0.8660254037844386),
                        Point(1., 1.7320508075688774),
                        Point(0., 1.7320508075688776),
                        Point(-0.5, 0.8660254037844393),
                    ],
                    parity: false,
                },
            ],
            proto_vertex_stars: vec![
                ProtoVertexStar {
                    index: 0,
                    proto_tiles: vec![
                        ProtoTile {
                            points: vec![
                                Point(0., 0.),
                                Point(1., 0.),
                                Point(1.5, 0.8660254037844386),
                                Point(1., 1.7320508075688774),
                                Point(0., 1.7320508075688776),
                                Point(-0.5, 0.8660254037844393),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0.5, 0.866025403784438),
                                Point(-1.5, 0.8660254037844369),
                                Point(-2., -0.),
                                Point(-1.5, -0.8660254037844404),
                                Point(-0.5, -0.8660254037844397),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0.5, -0.8660254037844397),
                                Point(0., -1.7320508075688772),
                                Point(1., -1.7320508075688754),
                                Point(1.5, -0.8660254037844358),
                                Point(1., 0.),
                            ],
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (1., 0.),
                                rotate: PI,
                            },
                            neighbor_index: 1,
                            forward_tile_index: 0,
                            reverse_tile_index: 2,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (-0.5, 0.866025403784438),
                                rotate: 5. / 3. * PI,
                            },
                            neighbor_index: 2,
                            forward_tile_index: 0,
                            reverse_tile_index: 2,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (-0.5, -0.8660254037844397),
                                rotate: 1. / 3. * PI,
                            },
                            neighbor_index: 0,
                            forward_tile_index: 0,
                            reverse_tile_index: 2,
                        },
                    ],
                },
            ],
        }
    }

    fn get_test_atlas_3_12_12() -> Atlas {
        Atlas {
            proto_tiles: vec![
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            1.0,
                            0.0,
                        ),
                        Point(
                            0.5000000000000002,
                            0.8660254037844388,
                        ),
                    ],
                    parity: false,
                },
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            0.5000000000000003,
                            0.8660254037844385,
                        ),
                        Point(
                            0.5000000000000009,
                            1.8660254037844388,
                        ),
                        Point(
                            0.0000000000000013322676295501878,
                            2.732050807568877,
                        ),
                        Point(
                            -0.8660254037844373,
                            3.2320508075688776,
                        ),
                        Point(
                            -1.8660254037844375,
                            3.2320508075688785,
                        ),
                        Point(
                            -2.7320508075688763,
                            2.7320508075688785,
                        ),
                        Point(
                            -3.2320508075688763,
                            1.8660254037844404,
                        ),
                        Point(
                            -3.232050807568877,
                            0.8660254037844403,
                        ),
                        Point(
                            -2.7320508075688776,
                            0.0000000000000013322676295501878,
                        ),
                        Point(
                            -1.8660254037844397,
                            -0.49999999999999933,
                        ),
                        Point(
                            -0.8660254037844396,
                            -0.4999999999999997,
                        ),
                    ],
                    parity: false,
                },
            ],
            proto_vertex_stars: vec![
                ProtoVertexStar {
                    index: 0,
                    proto_tiles: vec![
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    1.0,
                                    0.0,
                                ),
                                Point(
                                    0.5000000000000002,
                                    0.8660254037844388,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    0.5000000000000003,
                                    0.8660254037844385,
                                ),
                                Point(
                                    0.5000000000000009,
                                    1.8660254037844388,
                                ),
                                Point(
                                    0.0000000000000013322676295501878,
                                    2.732050807568877,
                                ),
                                Point(
                                    -0.8660254037844373,
                                    3.2320508075688776,
                                ),
                                Point(
                                    -1.8660254037844375,
                                    3.2320508075688785,
                                ),
                                Point(
                                    -2.7320508075688763,
                                    2.7320508075688785,
                                ),
                                Point(
                                    -3.2320508075688763,
                                    1.8660254037844404,
                                ),
                                Point(
                                    -3.232050807568877,
                                    0.8660254037844403,
                                ),
                                Point(
                                    -2.7320508075688776,
                                    0.0000000000000013322676295501878,
                                ),
                                Point(
                                    -1.8660254037844397,
                                    -0.49999999999999933,
                                ),
                                Point(
                                    -0.8660254037844396,
                                    -0.4999999999999997,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    -0.8660254037844386,
                                    -0.5000000000000001,
                                ),
                                Point(
                                    -1.3660254037844386,
                                    -1.3660254037844388,
                                ),
                                Point(
                                    -1.3660254037844386,
                                    -2.366025403784439,
                                ),
                                Point(
                                    -0.8660254037844386,
                                    -3.232050807568877,
                                ),
                                Point(
                                    0.0000000000000002220446049250313,
                                    -3.732050807568877,
                                ),
                                Point(
                                    1.0,
                                    -3.7320508075688776,
                                ),
                                Point(
                                    1.8660254037844386,
                                    -3.2320508075688776,
                                ),
                                Point(
                                    2.366025403784439,
                                    -2.366025403784439,
                                ),
                                Point(
                                    2.3660254037844393,
                                    -1.3660254037844388,
                                ),
                                Point(
                                    1.8660254037844393,
                                    -0.5000000000000002,
                                ),
                                Point(
                                    1.0000000000000007,
                                    -0.0000000000000002220446049250313,
                                ),
                            ],
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    1.0,
                                    0.0,
                                ),
                                rotate: 3.141592653589793,
                            },
                            neighbor_index: 1,
                            forward_tile_index: 0,
                            reverse_tile_index: 2,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    0.5000000000000003,
                                    0.8660254037844385,
                                ),
                                rotate: 4.18879020478639,
                            },
                            neighbor_index: 0,
                            forward_tile_index: 0,
                            reverse_tile_index: 2,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    -0.8660254037844386,
                                    -0.5000000000000001,
                                ),
                                rotate: 0.5235987755982991,
                            },
                            neighbor_index: 2,
                            forward_tile_index: 0,
                            reverse_tile_index: 2,
                        },
                    ],
                },
            ],
        }
    }

    fn get_test_atlas_4_6_12() -> Atlas {
        Atlas {
            proto_tiles: vec![
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            -0.8660254037844389,
                            0.49999999999999956,
                        ),
                        Point(
                            -1.7320508075688774,
                            -0.0000000000000008881784197001252,
                        ),
                        Point(
                            -1.7320508075688772,
                            -1.0000000000000009,
                        ),
                        Point(
                            -0.8660254037844383,
                            -1.5000000000000009,
                        ),
                        Point(
                            0.0000000000000003885780586188048,
                            -1.0000000000000007,
                        ),
                    ],
                    parity: false,
                },
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            1.0,
                            0.0,
                        ),
                        Point(
                            1.8660254037844388,
                            0.49999999999999994,
                        ),
                        Point(
                            2.366025403784439,
                            1.3660254037844386,
                        ),
                        Point(
                            2.366025403784439,
                            2.3660254037844384,
                        ),
                        Point(
                            1.866025403784439,
                            3.232050807568877,
                        ),
                        Point(
                            1.0000000000000004,
                            3.732050807568877,
                        ),
                        Point(
                            0.0000000000000004440892098500626,
                            3.732050807568877,
                        ),
                        Point(
                            -0.8660254037844384,
                            3.2320508075688776,
                        ),
                        Point(
                            -1.3660254037844388,
                            2.3660254037844393,
                        ),
                        Point(
                            -1.366025403784439,
                            1.3660254037844393,
                        ),
                        Point(
                            -0.866025403784439,
                            0.5000000000000007,
                        ),
                    ],
                    parity: false,
                },
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            0.0000000000000015926598195281475,
                            -1.0,
                        ),
                        Point(
                            1.0000000000000016,
                            -0.9999999999999984,
                        ),
                        Point(
                            1.0,
                            0.0000000000000015926598195281475,
                        ),
                    ],
                    parity: false,
                },
            ],
            proto_vertex_stars: vec![
                ProtoVertexStar {
                    index: 0,
                    proto_tiles: vec![
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    1.0,
                                    0.0,
                                ),
                                Point(
                                    1.8660254037844388,
                                    0.49999999999999994,
                                ),
                                Point(
                                    2.366025403784439,
                                    1.3660254037844386,
                                ),
                                Point(
                                    2.366025403784439,
                                    2.3660254037844384,
                                ),
                                Point(
                                    1.866025403784439,
                                    3.232050807568877,
                                ),
                                Point(
                                    1.0000000000000004,
                                    3.732050807568877,
                                ),
                                Point(
                                    0.0000000000000004440892098500626,
                                    3.732050807568877,
                                ),
                                Point(
                                    -0.8660254037844384,
                                    3.2320508075688776,
                                ),
                                Point(
                                    -1.3660254037844388,
                                    2.3660254037844393,
                                ),
                                Point(
                                    -1.366025403784439,
                                    1.3660254037844393,
                                ),
                                Point(
                                    -0.866025403784439,
                                    0.5000000000000007,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    -0.8660254037844389,
                                    0.49999999999999956,
                                ),
                                Point(
                                    -1.7320508075688774,
                                    -0.0000000000000008881784197001252,
                                ),
                                Point(
                                    -1.7320508075688772,
                                    -1.0000000000000009,
                                ),
                                Point(
                                    -0.8660254037844383,
                                    -1.5000000000000009,
                                ),
                                Point(
                                    0.0000000000000003885780586188048,
                                    -1.0000000000000007,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    0.0000000000000015926598195281475,
                                    -1.0,
                                ),
                                Point(
                                    1.0000000000000016,
                                    -0.9999999999999984,
                                ),
                                Point(
                                    1.0,
                                    0.0000000000000015926598195281475,
                                ),
                            ],
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: true,
                                translate: (
                                    1.0,
                                    0.0,
                                ),
                                rotate: 3.141592653589793,
                            },
                            neighbor_index: 0,
                            forward_tile_index: 0,
                            reverse_tile_index: 2,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: true,
                                translate: (
                                    -0.8660254037844389,
                                    0.49999999999999956,
                                ),
                                rotate: 5.759586531581288,
                            },
                            neighbor_index: 1,
                            forward_tile_index: 0,
                            reverse_tile_index: 2,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: true,
                                translate: (
                                    0.0000000000000015926598195281475,
                                    -1.0,
                                ),
                                rotate: 1.5707963267948983,
                            },
                            neighbor_index: 2,
                            forward_tile_index: 0,
                            reverse_tile_index: 2,
                        },
                    ],
                },
            ],
        }
    }

    fn get_test_atlas_4_6apio6_6aapio2_6apio6() -> Atlas {
        Atlas {
            proto_tiles: vec![
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            0.8660254037844389,
                            0.49999999999999944,
                        ),
                        Point(
                            0.3660254037844395,
                            1.3660254037844384,
                        ),
                        Point(
                            -0.49999999999999944,
                            0.8660254037844389,
                        ),
                    ],
                    parity: false,
                },
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            1.0,
                            0.0,
                        ),
                        Point(
                            1.8660254037844388,
                            -0.49999999999999983,
                        ),
                        Point(
                            2.366025403784439,
                            0.3660254037844389,
                        ),
                        Point(
                            3.232050807568877,
                            0.866025403784439,
                        ),
                        Point(
                            2.732050807568877,
                            1.7320508075688776,
                        ),
                        Point(
                            2.7320508075688767,
                            2.7320508075688776,
                        ),
                        Point(
                            1.7320508075688767,
                            2.732050807568877,
                        ),
                        Point(
                            0.8660254037844378,
                            3.2320508075688767,
                        ),
                        Point(
                            0.36602540378443815,
                            2.366025403784438,
                        ),
                        Point(
                            -0.5000000000000002,
                            1.8660254037844377,
                        ),
                        Point(
                            0.0000000000000005551115123125784,
                            0.9999999999999992,
                        ),
                    ],
                    parity: false,
                },
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            -0.866025403784438,
                            0.5000000000000012,
                        ),
                        Point(
                            -1.3660254037844395,
                            -0.3660254037844366,
                        ),
                        Point(
                            -2.2320508075688785,
                            -0.8660254037844354,
                        ),
                        Point(
                            -1.73205080756888,
                            -1.7320508075688748,
                        ),
                        Point(
                            -1.7320508075688812,
                            -2.732050807568875,
                        ),
                        Point(
                            -0.7320508075688813,
                            -2.732050807568876,
                        ),
                        Point(
                            0.13397459621555682,
                            -3.2320508075688767,
                        ),
                        Point(
                            0.633974596215558,
                            -2.366025403784439,
                        ),
                        Point(
                            1.4999999999999971,
                            -1.8660254037844402,
                        ),
                        Point(
                            0.9999999999999978,
                            -1.0000000000000009,
                        ),
                        Point(
                            1.0,
                            -0.0000000000000016538921594855151,
                        ),
                    ],
                    parity: false,
                },
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            -0.00000000000000012246467991473532,
                            1.0,
                        ),
                        Point(
                            -0.5,
                            0.13397459621556124,
                        ),
                        Point(
                            -1.3660254037844388,
                            0.633974596215561,
                        ),
                        Point(
                            -0.8660254037844388,
                            -0.23205080756887755,
                        ),
                        Point(
                            -1.7320508075688772,
                            -0.7320508075688779,
                        ),
                        Point(
                            -0.7320508075688771,
                            -0.7320508075688779,
                        ),
                        Point(
                            -0.732050807568877,
                            -1.7320508075688776,
                        ),
                        Point(
                            -0.23205080756887753,
                            -0.8660254037844388,
                        ),
                        Point(
                            0.6339745962155615,
                            -1.3660254037844384,
                        ),
                        Point(
                            0.13397459621556057,
                            -0.5000000000000002,
                        ),
                        Point(
                            1.0,
                            0.00000000000000012246467991473532,
                        ),
                    ],
                    parity: false,
                },
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            1.0,
                            0.0,
                        ),
                        Point(
                            1.0,
                            -1.0,
                        ),
                        Point(
                            1.5,
                            -0.1339745962155613,
                        ),
                        Point(
                            2.366025403784439,
                            -0.6339745962155612,
                        ),
                        Point(
                            1.8660254037844388,
                            0.23205080756887744,
                        ),
                        Point(
                            2.732050807568877,
                            0.7320508075688776,
                        ),
                        Point(
                            1.7320508075688772,
                            0.7320508075688777,
                        ),
                        Point(
                            1.7320508075688772,
                            1.7320508075688776,
                        ),
                        Point(
                            1.2320508075688776,
                            0.8660254037844388,
                        ),
                        Point(
                            0.3660254037844387,
                            1.3660254037844384,
                        ),
                        Point(
                            0.8660254037844395,
                            0.5000000000000002,
                        ),
                    ],
                    parity: false,
                },
            ],
            proto_vertex_stars: vec![
                ProtoVertexStar {
                    index: 0,
                    proto_tiles: vec![
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    1.0,
                                    0.0,
                                ),
                                Point(
                                    1.8660254037844388,
                                    -0.49999999999999983,
                                ),
                                Point(
                                    2.366025403784439,
                                    0.3660254037844389,
                                ),
                                Point(
                                    3.232050807568877,
                                    0.866025403784439,
                                ),
                                Point(
                                    2.732050807568877,
                                    1.7320508075688776,
                                ),
                                Point(
                                    2.7320508075688767,
                                    2.7320508075688776,
                                ),
                                Point(
                                    1.7320508075688767,
                                    2.732050807568877,
                                ),
                                Point(
                                    0.8660254037844378,
                                    3.2320508075688767,
                                ),
                                Point(
                                    0.36602540378443815,
                                    2.366025403784438,
                                ),
                                Point(
                                    -0.5000000000000002,
                                    1.8660254037844377,
                                ),
                                Point(
                                    0.0000000000000005551115123125784,
                                    0.9999999999999992,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    -0.00000000000000012246467991473532,
                                    1.0,
                                ),
                                Point(
                                    -0.5,
                                    0.13397459621556124,
                                ),
                                Point(
                                    -1.3660254037844388,
                                    0.633974596215561,
                                ),
                                Point(
                                    -0.8660254037844388,
                                    -0.23205080756887755,
                                ),
                                Point(
                                    -1.7320508075688772,
                                    -0.7320508075688779,
                                ),
                                Point(
                                    -0.7320508075688771,
                                    -0.7320508075688779,
                                ),
                                Point(
                                    -0.732050807568877,
                                    -1.7320508075688776,
                                ),
                                Point(
                                    -0.23205080756887753,
                                    -0.8660254037844388,
                                ),
                                Point(
                                    0.6339745962155615,
                                    -1.3660254037844384,
                                ),
                                Point(
                                    0.13397459621556057,
                                    -0.5000000000000002,
                                ),
                                Point(
                                    1.0,
                                    0.00000000000000012246467991473532,
                                ),
                            ],
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 1,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    1.0,
                                    0.0,
                                ),
                                rotate: 3.141592653589793,
                            },
                            neighbor_index: 0,
                            forward_tile_index: 0,
                            reverse_tile_index: 1,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 1,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    -0.00000000000000012246467991473532,
                                    1.0,
                                ),
                                rotate: 4.188790204786391,
                            },
                            neighbor_index: 3,
                            forward_tile_index: 0,
                            reverse_tile_index: 1,
                        },
                    ],
                },
                ProtoVertexStar {
                    index: 1,
                    proto_tiles: vec![
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    1.0,
                                    0.0,
                                ),
                                Point(
                                    1.0,
                                    -1.0,
                                ),
                                Point(
                                    1.5,
                                    -0.1339745962155613,
                                ),
                                Point(
                                    2.366025403784439,
                                    -0.6339745962155612,
                                ),
                                Point(
                                    1.8660254037844388,
                                    0.23205080756887744,
                                ),
                                Point(
                                    2.732050807568877,
                                    0.7320508075688776,
                                ),
                                Point(
                                    1.7320508075688772,
                                    0.7320508075688777,
                                ),
                                Point(
                                    1.7320508075688772,
                                    1.7320508075688776,
                                ),
                                Point(
                                    1.2320508075688776,
                                    0.8660254037844388,
                                ),
                                Point(
                                    0.3660254037844387,
                                    1.3660254037844384,
                                ),
                                Point(
                                    0.8660254037844395,
                                    0.5000000000000002,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    0.8660254037844389,
                                    0.49999999999999944,
                                ),
                                Point(
                                    0.3660254037844395,
                                    1.3660254037844384,
                                ),
                                Point(
                                    -0.49999999999999944,
                                    0.8660254037844389,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    -0.49999999999999944,
                                    0.866025403784439,
                                ),
                                Point(
                                    0.3660254037844396,
                                    1.3660254037844384,
                                ),
                                Point(
                                    -0.6339745962155605,
                                    1.366025403784439,
                                ),
                                Point(
                                    -0.63397459621556,
                                    2.3660254037844393,
                                ),
                                Point(
                                    -1.1339745962155607,
                                    1.5000000000000009,
                                ),
                                Point(
                                    -1.9999999999999991,
                                    2.0000000000000013,
                                ),
                                Point(
                                    -1.4999999999999996,
                                    1.133974596215562,
                                ),
                                Point(
                                    -2.3660254037844384,
                                    0.6339745962155628,
                                ),
                                Point(
                                    -1.3660254037844386,
                                    0.6339745962155625,
                                ),
                                Point(
                                    -1.3660254037844388,
                                    -0.36602540378443754,
                                ),
                                Point(
                                    -0.8660254037844389,
                                    0.5000000000000013,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    -0.866025403784438,
                                    0.5000000000000012,
                                ),
                                Point(
                                    -1.3660254037844395,
                                    -0.3660254037844366,
                                ),
                                Point(
                                    -2.2320508075688785,
                                    -0.8660254037844354,
                                ),
                                Point(
                                    -1.73205080756888,
                                    -1.7320508075688748,
                                ),
                                Point(
                                    -1.7320508075688812,
                                    -2.732050807568875,
                                ),
                                Point(
                                    -0.7320508075688813,
                                    -2.732050807568876,
                                ),
                                Point(
                                    0.13397459621555682,
                                    -3.2320508075688767,
                                ),
                                Point(
                                    0.633974596215558,
                                    -2.366025403784439,
                                ),
                                Point(
                                    1.4999999999999971,
                                    -1.8660254037844402,
                                ),
                                Point(
                                    0.9999999999999978,
                                    -1.0000000000000009,
                                ),
                                Point(
                                    1.0,
                                    -0.0000000000000016538921594855151,
                                ),
                            ],
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    1.0,
                                    0.0,
                                ),
                                rotate: 3.141592653589793,
                            },
                            neighbor_index: 0,
                            forward_tile_index: 1,
                            reverse_tile_index: 0,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 2,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    0.8660254037844389,
                                    0.49999999999999944,
                                ),
                                rotate: 3.6651914291880914,
                            },
                            neighbor_index: 1,
                            forward_tile_index: 1,
                            reverse_tile_index: 0,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 2,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    -0.49999999999999944,
                                    0.866025403784439,
                                ),
                                rotate: 5.235987755982988,
                            },
                            neighbor_index: 0,
                            forward_tile_index: 1,
                            reverse_tile_index: 0,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    -0.866025403784438,
                                    0.5000000000000012,
                                ),
                                rotate: 4.18879020478639,
                            },
                            neighbor_index: 1,
                            forward_tile_index: 1,
                            reverse_tile_index: 0,
                        },
                    ],
                },
                ProtoVertexStar {
                    index: 2,
                    proto_tiles: vec![
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    1.0,
                                    0.0,
                                ),
                                Point(
                                    1.0,
                                    1.0,
                                ),
                                Point(
                                    0.0,
                                    1.0,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    -0.00000000000000012246467991473532,
                                    1.0,
                                ),
                                Point(
                                    -0.5,
                                    0.13397459621556124,
                                ),
                                Point(
                                    -1.3660254037844388,
                                    0.633974596215561,
                                ),
                                Point(
                                    -0.8660254037844388,
                                    -0.23205080756887755,
                                ),
                                Point(
                                    -1.7320508075688772,
                                    -0.7320508075688779,
                                ),
                                Point(
                                    -0.7320508075688771,
                                    -0.7320508075688779,
                                ),
                                Point(
                                    -0.732050807568877,
                                    -1.7320508075688776,
                                ),
                                Point(
                                    -0.23205080756887753,
                                    -0.8660254037844388,
                                ),
                                Point(
                                    0.6339745962155615,
                                    -1.3660254037844384,
                                ),
                                Point(
                                    0.13397459621556057,
                                    -0.5000000000000002,
                                ),
                                Point(
                                    1.0,
                                    0.00000000000000012246467991473532,
                                ),
                            ],
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 1,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    1.0,
                                    0.0,
                                ),
                                rotate: 3.141592653589793,
                            },
                            neighbor_index: 2,
                            forward_tile_index: 2,
                            reverse_tile_index: 1,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 1,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    -0.00000000000000012246467991473532,
                                    1.0,
                                ),
                                rotate: 4.188790204786391,
                            },
                            neighbor_index: 1,
                            forward_tile_index: 2,
                            reverse_tile_index: 1,
                        },
                    ],
                },
            ],
        }
    }

    fn get_test_atlas_6_4apio6_6_4apio6() -> Atlas {
        Atlas {
            proto_tiles: vec![
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            1.0,
                            0.0,
                        ),
                        Point(
                            1.5,
                            -0.8660254037844387,
                        ),
                        Point(
                            1.5,
                            0.1339745962155613,
                        ),
                        Point(
                            2.366025403784439,
                            0.6339745962155612,
                        ),
                        Point(
                            1.3660254037844388,
                            0.6339745962155613,
                        ),
                        Point(
                            0.866025403784439,
                            1.5,
                        ),
                        Point(
                            0.8660254037844388,
                            0.5,
                        ),
                    ],
                    parity: false,
                },
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            0.8660254037844389,
                            0.49999999999999944,
                        ),
                        Point(
                            0.8660254037844397,
                            1.4999999999999993,
                        ),
                        Point(
                            0.0000000000000014432899320127035,
                            2.0,
                        ),
                        Point(
                            -0.8660254037844376,
                            1.5000000000000009,
                        ),
                        Point(
                            -0.8660254037844388,
                            0.5000000000000009,
                        ),
                    ],
                    parity: false,
                },
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            -1.0,
                            0.00000000000000012246467991473532,
                        ),
                        Point(
                            -1.5,
                            -0.8660254037844385,
                        ),
                        Point(
                            -2.0,
                            -1.7320508075688772,
                        ),
                        Point(
                            -1.5000000000000002,
                            -2.598076211353316,
                        ),
                        Point(
                            -1.0000000000000004,
                            -3.4641016151377544,
                        ),
                        Point(
                            -0.00000000000000042423009548996267,
                            -3.464101615137754,
                        ),
                        Point(
                            0.9999999999999996,
                            -3.4641016151377535,
                        ),
                        Point(
                            1.4999999999999993,
                            -2.5980762113533147,
                        ),
                        Point(
                            1.999999999999999,
                            -1.7320508075688763,
                        ),
                        Point(
                            1.4999999999999987,
                            -0.8660254037844379,
                        ),
                        Point(
                            1.0,
                            -0.00000000000000012246467991473532,
                        ),
                    ],
                    parity: false,
                },
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            1.0,
                            0.0,
                        ),
                        Point(
                            2.0,
                            0.0,
                        ),
                        Point(
                            2.5,
                            0.8660254037844387,
                        ),
                        Point(
                            3.0,
                            1.7320508075688774,
                        ),
                        Point(
                            2.5,
                            2.598076211353316,
                        ),
                        Point(
                            2.0,
                            3.4641016151377544,
                        ),
                        Point(
                            1.0,
                            3.464101615137754,
                        ),
                        Point(
                            0.0,
                            3.4641016151377535,
                        ),
                        Point(
                            -0.49999999999999967,
                            2.5980762113533147,
                        ),
                        Point(
                            -0.9999999999999992,
                            1.732050807568876,
                        ),
                        Point(
                            -0.49999999999999856,
                            0.8660254037844377,
                        ),
                    ],
                    parity: false,
                },
                ProtoTile {
                    points: vec![
                        Point(
                            0.0,
                            0.0,
                        ),
                        Point(
                            -0.5000000000000001,
                            0.8660254037844386,
                        ),
                        Point(
                            -0.5,
                            -0.13397459621556135,
                        ),
                        Point(
                            -1.3660254037844388,
                            -0.6339745962155614,
                        ),
                        Point(
                            -0.36602540378443876,
                            -0.6339745962155613,
                        ),
                        Point(
                            0.13397459621556115,
                            -1.5,
                        ),
                        Point(
                            0.13397459621556124,
                            -0.5,
                        ),
                        Point(
                            1.0,
                            0.00000000000000012246467991473532,
                        ),
                    ],
                    parity: false,
                },
            ],
            proto_vertex_stars: vec![
                ProtoVertexStar {
                    index: 0,
                    proto_tiles: vec![
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    1.0,
                                    0.0,
                                ),
                                Point(
                                    1.5,
                                    -0.8660254037844387,
                                ),
                                Point(
                                    1.5,
                                    0.1339745962155613,
                                ),
                                Point(
                                    2.366025403784439,
                                    0.6339745962155612,
                                ),
                                Point(
                                    1.3660254037844388,
                                    0.6339745962155613,
                                ),
                                Point(
                                    0.866025403784439,
                                    1.5,
                                ),
                                Point(
                                    0.8660254037844388,
                                    0.5,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    0.8660254037844389,
                                    0.49999999999999944,
                                ),
                                Point(
                                    0.8660254037844397,
                                    1.4999999999999993,
                                ),
                                Point(
                                    0.0000000000000014432899320127035,
                                    2.0,
                                ),
                                Point(
                                    -0.8660254037844376,
                                    1.5000000000000009,
                                ),
                                Point(
                                    -0.8660254037844388,
                                    0.5000000000000009,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    -0.8660254037844389,
                                    0.49999999999999956,
                                ),
                                Point(
                                    -0.8660254037844395,
                                    1.4999999999999996,
                                ),
                                Point(
                                    -1.366025403784439,
                                    0.6339745962155607,
                                ),
                                Point(
                                    -2.3660254037844393,
                                    0.6339745962155603,
                                ),
                                Point(
                                    -1.5000000000000002,
                                    0.13397459621556074,
                                ),
                                Point(
                                    -1.5,
                                    -0.8660254037844393,
                                ),
                                Point(
                                    -1.0000000000000002,
                                    -0.0000000000000004440892098500626,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    -1.0,
                                    0.00000000000000012246467991473532,
                                ),
                                Point(
                                    -1.5,
                                    -0.8660254037844385,
                                ),
                                Point(
                                    -2.0,
                                    -1.7320508075688772,
                                ),
                                Point(
                                    -1.5000000000000002,
                                    -2.598076211353316,
                                ),
                                Point(
                                    -1.0000000000000004,
                                    -3.4641016151377544,
                                ),
                                Point(
                                    -0.00000000000000042423009548996267,
                                    -3.464101615137754,
                                ),
                                Point(
                                    0.9999999999999996,
                                    -3.4641016151377535,
                                ),
                                Point(
                                    1.4999999999999993,
                                    -2.5980762113533147,
                                ),
                                Point(
                                    1.999999999999999,
                                    -1.7320508075688763,
                                ),
                                Point(
                                    1.4999999999999987,
                                    -0.8660254037844379,
                                ),
                                Point(
                                    1.0,
                                    -0.00000000000000012246467991473532,
                                ),
                            ],
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 1,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    1.0,
                                    0.0,
                                ),
                                rotate: 3.141592653589793,
                            },
                            neighbor_index: 0,
                            forward_tile_index: 0,
                            reverse_tile_index: 3,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 2,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    0.8660254037844389,
                                    0.49999999999999944,
                                ),
                                rotate: 3.6651914291880914,
                            },
                            neighbor_index: 1,
                            forward_tile_index: 0,
                            reverse_tile_index: 3,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 2,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    -0.8660254037844389,
                                    0.49999999999999956,
                                ),
                                rotate: 5.759586531581288,
                            },
                            neighbor_index: 0,
                            forward_tile_index: 0,
                            reverse_tile_index: 3,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 1,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    -1.0,
                                    0.00000000000000012246467991473532,
                                ),
                                rotate: 4.18879020478639,
                            },
                            neighbor_index: 1,
                            forward_tile_index: 0,
                            reverse_tile_index: 3,
                        },
                    ],
                },
                ProtoVertexStar {
                    index: 1,
                    proto_tiles: vec![
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    1.0,
                                    0.0,
                                ),
                                Point(
                                    2.0,
                                    0.0,
                                ),
                                Point(
                                    2.5,
                                    0.8660254037844387,
                                ),
                                Point(
                                    3.0,
                                    1.7320508075688774,
                                ),
                                Point(
                                    2.5,
                                    2.598076211353316,
                                ),
                                Point(
                                    2.0,
                                    3.4641016151377544,
                                ),
                                Point(
                                    1.0,
                                    3.464101615137754,
                                ),
                                Point(
                                    0.0,
                                    3.4641016151377535,
                                ),
                                Point(
                                    -0.49999999999999967,
                                    2.5980762113533147,
                                ),
                                Point(
                                    -0.9999999999999992,
                                    1.732050807568876,
                                ),
                                Point(
                                    -0.49999999999999856,
                                    0.8660254037844377,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    -0.5000000000000001,
                                    0.8660254037844386,
                                ),
                                Point(
                                    -0.5,
                                    -0.13397459621556135,
                                ),
                                Point(
                                    -1.3660254037844388,
                                    -0.6339745962155614,
                                ),
                                Point(
                                    -0.36602540378443876,
                                    -0.6339745962155613,
                                ),
                                Point(
                                    0.13397459621556115,
                                    -1.5,
                                ),
                                Point(
                                    0.13397459621556124,
                                    -0.5,
                                ),
                                Point(
                                    1.0,
                                    0.00000000000000012246467991473532,
                                ),
                            ],
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    1.0,
                                    0.0,
                                ),
                                rotate: 3.141592653589793,
                            },
                            neighbor_index: 0,
                            forward_tile_index: 1,
                            reverse_tile_index: 0,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    -0.5000000000000001,
                                    0.8660254037844386,
                                ),
                                rotate: 4.7123889803846915,
                            },
                            neighbor_index: 3,
                            forward_tile_index: 1,
                            reverse_tile_index: 0,
                        },
                    ],
                },
                ProtoVertexStar {
                    index: 2,
                    proto_tiles: vec![
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    1.0,
                                    0.0,
                                ),
                                Point(
                                    1.5,
                                    0.8660254037844386,
                                ),
                                Point(
                                    1.0000000000000002,
                                    1.7320508075688774,
                                ),
                                Point(
                                    0.0000000000000002220446049250313,
                                    1.7320508075688776,
                                ),
                                Point(
                                    -0.5000000000000002,
                                    0.8660254037844393,
                                ),
                            ],
                            parity: false,
                        },
                        ProtoTile {
                            points: vec![
                                Point(
                                    0.0,
                                    0.0,
                                ),
                                Point(
                                    -0.5000000000000017,
                                    0.8660254037844377,
                                ),
                                Point(
                                    -0.4999999999999997,
                                    -0.13397459621556224,
                                ),
                                Point(
                                    -1.3660254037844377,
                                    -0.6339745962155637,
                                ),
                                Point(
                                    -0.3660254037844376,
                                    -0.633974596215562,
                                ),
                                Point(
                                    0.13397459621556382,
                                    -1.4999999999999998,
                                ),
                                Point(
                                    0.13397459621556213,
                                    -0.4999999999999997,
                                ),
                                Point(
                                    1.0,
                                    0.0000000000000018988215193149856,
                                ),
                            ],
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    1.0,
                                    0.0,
                                ),
                                rotate: 3.141592653589793,
                            },
                            neighbor_index: 2,
                            forward_tile_index: 2,
                            reverse_tile_index: 1,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (
                                    -0.5000000000000017,
                                    0.8660254037844377,
                                ),
                                rotate: 4.7123889803846915,
                            },
                            neighbor_index: 1,
                            forward_tile_index: 2,
                            reverse_tile_index: 1,
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn test_atlas_6_4apio6_6_4apio6() {
        let _patch = match Patch::new(
            get_test_atlas_6_4apio6_6_4apio6(),
            get_tile_tree_config(),
            get_vertex_star_tree_config(),
        ) {
            Ok(_) => {},
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            },
        };
    }

    // TODO: add 4_6apio6_6aapio2_6apio6 to the list of all test atlases
    fn get_all_test_atlases() -> [Atlas; 5] {
        [
            get_test_atlas_4_4_4_4(),
            get_test_atlas_3_3_3_3_3_3(),
            get_test_atlas_6_6_6(),
            get_test_atlas_3_12_12(),
            get_test_atlas_4_6_12(),
        ]
    }

    #[test]
    // by link refers to asserting that all of a vertex star's neighbors (i.e. the vertex star's link) are correctly configured
    fn test_vertex_star_get_neighbor_vertex_star_by_link() {
        let [
            atlas_4_4_4_4,
            atlas_3_3_3_3_3_3,
            atlas_6_6_6,
            atlas_3_12_12,
            atlas_4_6_12,
        ] = get_all_test_atlases();

        for rotation in (0..8).map(|i| rad((i as f64) * TAU / 8.)) {
            println!("rotation: {}", fmt_float(rotation / TAU * 360., 2));

            let rotate = Euclid::Rotate(rotation);

            let assert_vertex_star_neighbor = |
                atlas: &Atlas,
                vertex_star: &VertexStar,
                neighbor_index: usize,
                expected_point: Point,
                expected_parity: bool,
                expected_rotation: f64,
            | {
                println!("input: {} {} | expected: {} {} {}π", vertex_star.point, neighbor_index, expected_point.transform(&rotate), expected_parity, fmt_float(rad(expected_rotation + rotation) / PI, 2));
                let neighbor_vertex_star = vertex_star.get_neighbor_vertex_star(atlas, neighbor_index).unwrap();
                assert_eq!(expected_point.transform(&rotate), neighbor_vertex_star.point);
                assert_eq!(expected_parity, neighbor_vertex_star.parity);
                approx_eq!(f64, rad(expected_rotation + rotation), neighbor_vertex_star.rotation);
            };

            println!("Atlas 4.4.4.4");
            let vertex_star = VertexStar::new(&atlas_4_4_4_4, Point(0.,0.), 0, false, rotation);
            assert_vertex_star_neighbor(&atlas_4_4_4_4, &vertex_star, 0, X.transform(&Euclid::Rotate(to_rad(0. * 90.))), false, 0.);
            assert_vertex_star_neighbor(&atlas_4_4_4_4, &vertex_star, 1, X.transform(&Euclid::Rotate(to_rad(1. * 90.))), false, 0.);
            assert_vertex_star_neighbor(&atlas_4_4_4_4, &vertex_star, 2, X.transform(&Euclid::Rotate(to_rad(2. * 90.))), false, 0.);
            assert_vertex_star_neighbor(&atlas_4_4_4_4, &vertex_star, 3, X.transform(&Euclid::Rotate(to_rad(3. * 90.))), false, 0.);
            println!();

            println!("Atlas 3.3.3.3.3.3");
            let vertex_star = VertexStar::new(&atlas_3_3_3_3_3_3, Point(0.,0.), 0, false, rotation);
            assert_vertex_star_neighbor(&atlas_3_3_3_3_3_3, &vertex_star, 0, X.transform(&Euclid::Rotate(to_rad(0. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&atlas_3_3_3_3_3_3, &vertex_star, 1, X.transform(&Euclid::Rotate(to_rad(1. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&atlas_3_3_3_3_3_3, &vertex_star, 2, X.transform(&Euclid::Rotate(to_rad(2. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&atlas_3_3_3_3_3_3, &vertex_star, 3, X.transform(&Euclid::Rotate(to_rad(3. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&atlas_3_3_3_3_3_3, &vertex_star, 4, X.transform(&Euclid::Rotate(to_rad(4. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&atlas_3_3_3_3_3_3, &vertex_star, 5, X.transform(&Euclid::Rotate(to_rad(5. * 60.))), false, 0.);
            println!();

            println!("Atlas 6.6.6");
            let vertex_star = VertexStar::new(&atlas_6_6_6, Point(0.,0.), 0, false, rotation);
            assert_vertex_star_neighbor(&atlas_6_6_6, &vertex_star, 0, X.transform(&Euclid::Rotate(to_rad(0. * 120.))), false, to_rad(60.));
            assert_vertex_star_neighbor(&atlas_6_6_6, &vertex_star, 1, X.transform(&Euclid::Rotate(to_rad(1. * 120.))), false, to_rad(60.));
            assert_vertex_star_neighbor(&atlas_6_6_6, &vertex_star, 2, X.transform(&Euclid::Rotate(to_rad(2. * 120.))), false, to_rad(60.));
            println!();

            println!("Atlas 3.12.12");
            let vertex_star = VertexStar::new(&atlas_3_12_12, Point(0.,0.), 0, false, rotation);
            assert_vertex_star_neighbor(&atlas_3_12_12, &vertex_star, 0, X, false, to_rad(120.));
            assert_vertex_star_neighbor(&atlas_3_12_12, &vertex_star, 1, X.transform(&Euclid::Rotate(to_rad(60.))), false, to_rad(240.));
            assert_vertex_star_neighbor(&atlas_3_12_12, &vertex_star, 2, X.transform(&Euclid::Rotate(to_rad(210.))), false, to_rad(180.));
            println!();

            println!("Atlas 4.6.12");
            let vertex_star = VertexStar::new(&atlas_4_6_12, Point(0.,0.), 0, false, rotation);
            assert_vertex_star_neighbor(&atlas_4_6_12, &vertex_star, 0, X, true, to_rad(180.));
            assert_vertex_star_neighbor(&atlas_4_6_12, &vertex_star, 1, X.transform(&Euclid::Rotate(to_rad(150.))), true, to_rad(120.));
            assert_vertex_star_neighbor(&atlas_4_6_12, &vertex_star, 2, X.transform(&Euclid::Rotate(to_rad(270.))), true, to_rad(0.));
            println!();
        }
    }

    #[test]
    // by chain refers to asserting that a sequence of vertex stars, the next accumulated as a neighbor of the previous star, are correctly configured
    fn test_vertex_star_get_neighbor_vertex_star_by_sequence() {
        let [
            atlas_4_4_4_4,
            atlas_3_3_3_3_3_3,
            atlas_6_6_6,
            atlas_3_12_12,
            atlas_4_6_12,
        ] = get_all_test_atlases();

        for rotation in (0..8).map(|i| rad((i as f64) * TAU / 8.)) {
            println!("rotation: {}", fmt_float(rotation / TAU * 360., 2));

            let rotate = Euclid::Rotate(rotation);

            let assert_vertex_star_neighbor = |
                atlas: &Atlas,
                vertex_star: &VertexStar,
                neighbor_index: usize,
                relative_expected_point: Point,
                relative_expected_parity: bool,
                relative_expected_rotation: f64,
            | -> VertexStar {
                let expected_point = &vertex_star.point + &relative_expected_point.transform(&Euclid::Rotate(vertex_star.rotation));
                let expected_parity = vertex_star.mutual_parity(relative_expected_parity);
                let expected_rotation = rad(vertex_star.rotation + relative_expected_rotation);

                println!("input: {} {} | expected: {} {} {}π", vertex_star.point, neighbor_index, expected_point, expected_parity, fmt_float(expected_rotation / PI, 2));

                let neighbor_vertex_star = vertex_star.get_neighbor_vertex_star(atlas, neighbor_index).unwrap();

                assert_eq!(expected_parity, neighbor_vertex_star.parity);
                assert_eq!(expected_point, neighbor_vertex_star.point);
                approx_eq!(f64, expected_rotation, neighbor_vertex_star.rotation);

                neighbor_vertex_star
            };

            println!("Atlas 4.4.4.4");
            let vertex_star = VertexStar::new(&atlas_4_4_4_4, Point(0.,0.), 0, false, rotation);
            let nvs = assert_vertex_star_neighbor(&atlas_4_4_4_4, &vertex_star, 0, X.transform(&Euclid::Rotate(to_rad(0. * 90.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&atlas_4_4_4_4, &nvs, 1, X.transform(&Euclid::Rotate(to_rad(1. * 90.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&atlas_4_4_4_4, &nvs, 2, X.transform(&Euclid::Rotate(to_rad(2. * 90.))), false, 0.);
            let _nvs = assert_vertex_star_neighbor(&atlas_4_4_4_4, &nvs, 3, X.transform(&Euclid::Rotate(to_rad(3. * 90.))), false, 0.);
            println!();

            println!("Atlas 3.3.3.3.3.3");
            let vertex_star = VertexStar::new(&atlas_3_3_3_3_3_3, Point(0.,0.), 0, false, rotation);
            let nvs = assert_vertex_star_neighbor(&atlas_3_3_3_3_3_3, &vertex_star, 0, X.transform(&Euclid::Rotate(to_rad(0. * 60.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&atlas_3_3_3_3_3_3, &nvs, 1, X.transform(&Euclid::Rotate(to_rad(1. * 60.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&atlas_3_3_3_3_3_3, &nvs, 2, X.transform(&Euclid::Rotate(to_rad(2. * 60.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&atlas_3_3_3_3_3_3, &nvs, 3, X.transform(&Euclid::Rotate(to_rad(3. * 60.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&atlas_3_3_3_3_3_3, &nvs, 4, X.transform(&Euclid::Rotate(to_rad(4. * 60.))), false, 0.);
            let _nvs = assert_vertex_star_neighbor(&atlas_3_3_3_3_3_3, &nvs, 5, X.transform(&Euclid::Rotate(to_rad(5. * 60.))), false, 0.);
            println!();

            println!("Atlas 6.6.6");
            let vertex_star = VertexStar::new(&atlas_6_6_6, Point(0.,0.), 0, false, rotation);
            let nvs = assert_vertex_star_neighbor(&atlas_6_6_6, &vertex_star, 0, X.transform(&Euclid::Rotate(to_rad(0. * 120.))), false, to_rad(60.));
            let nvs = assert_vertex_star_neighbor(&atlas_6_6_6, &nvs, 1, X.transform(&Euclid::Rotate(to_rad(1. * 120.))), false, to_rad(60.));
            let _nvs = assert_vertex_star_neighbor(&atlas_6_6_6, &nvs, 2, X.transform(&Euclid::Rotate(to_rad(2. * 120.))), false, to_rad(60.));
            println!();

            println!("Atlas 3.12.12");
            let vertex_star = VertexStar::new(&atlas_3_12_12, Point(0.,0.), 0, false, rotation);
            let nvs = assert_vertex_star_neighbor(&atlas_3_12_12, &vertex_star, 0, X, false, to_rad(120.));
            let nvs = assert_vertex_star_neighbor(&atlas_3_12_12, &nvs, 1, X.transform(&Euclid::Rotate(to_rad(60.))), false, to_rad(240.));
            let _nvs = assert_vertex_star_neighbor(&atlas_3_12_12, &nvs, 2, X.transform(&Euclid::Rotate(to_rad(210.))), false, to_rad(180.));
            println!();

            println!("Atlas 4.6.12");
            let vertex_star = VertexStar::new(&atlas_4_6_12, Point(0.,0.), 0, false, rotation);

            let nvs = vertex_star.get_neighbor_vertex_star(&atlas_4_6_12, 0).unwrap();
            assert_eq!(true, nvs.parity);
            assert_eq!(X.transform(&rotate), nvs.point);
            approx_eq!(f64, rad(to_rad(180.) + rotation), nvs.rotation);

            let nvs = nvs.get_neighbor_vertex_star(&atlas_4_6_12, 1).unwrap();
            assert_eq!(false, nvs.parity);
            assert_eq!((&X + &X.transform(&Euclid::Rotate(to_rad(30.)))).transform(&rotate), nvs.point);
            approx_eq!(f64, rad(to_rad(60.) + rotation), nvs.rotation);

            let nvs = nvs.get_neighbor_vertex_star(&atlas_4_6_12, 2).unwrap();
            assert_eq!(true, nvs.parity);
            assert_eq!((
                &(&X + &X.transform(&Euclid::Rotate(to_rad(30.)))) + &X.transform(&Euclid::Rotate(to_rad(-30.)))
            ).transform(&rotate), nvs.point);
            approx_eq!(f64, rad(to_rad(60.) + rotation), nvs.rotation);

            println!();
        }
    }

    #[test]
    fn test_vertex_star_get_proto_vertex_star() {
        let atlas = get_test_atlas_4_4_4_4();

        let proto_vertex_star_index = 0;
        let vertex_star = VertexStar::new(&atlas, Point(0.,0.), 0, false, 0.);
        let proto_vertex_star = vertex_star.get_proto_vertex_star(&atlas).unwrap();
        assert_eq!(proto_vertex_star.index, proto_vertex_star_index);
    }

    #[test]
    fn test_vertex_star_get_tile_4_6_12() {
        let atlas = get_test_atlas_4_6_12();

        let vertex_star = VertexStar::new(&atlas, ORIGIN.clone(), 0, false, 0.);

        let tile = vertex_star.get_tile(&atlas, &X).unwrap();
        assert_eq!(4, tile.size());
        assert_eq!(Point(0.5, -0.5), tile.centroid);

        let tile = vertex_star.get_tile(&atlas, &X.transform(&Euclid::Rotate(to_rad(150.)))).unwrap();
        assert_eq!(12, tile.size());
        assert_eq!(Point(0.5, 1. + 3_f64.sqrt() / 2.), tile.centroid);

        let tile = vertex_star.get_tile(&atlas, &(-Y)).unwrap();
        assert_eq!(6, tile.size());
        assert_eq!(Point(- 3_f64.sqrt() / 2., -0.5), tile.centroid);

        // flipped vertex star
        let vertex_star = VertexStar::new(&atlas, ORIGIN, 0, true, 0.);

        let tile = vertex_star.get_tile(&atlas, &X).unwrap();
        assert_eq!(12, tile.size());
        assert_eq!(Point(0.5, -(1. + 3_f64.sqrt() / 2.)), tile.centroid);

        let tile = vertex_star.get_tile(&atlas, &Y).unwrap();
        assert_eq!(4, tile.size());
        assert_eq!(Point(0.5, 0.5), tile.centroid);

        let tile = vertex_star.get_tile(&atlas, &X.transform(&Euclid::Rotate(to_rad(360. - 150.)))).unwrap();
        assert_eq!(6, tile.size());
        assert_eq!(Point(- 3_f64.sqrt() / 2., 0.5), tile.centroid);
    }

    #[test]
    fn test_vertex_star_get_tile_6_6_6() {
        let atlas = get_test_atlas_6_6_6();

        let vertex_star = VertexStar::new(&atlas, X, 0, false, to_rad(60.));

        let tile = vertex_star.get_tile(&atlas, &ORIGIN).unwrap();
        assert_eq!(6, tile.size());
        assert_eq!(Point(0.5, 3_f64.sqrt() / 2.), tile.centroid);

        let tile = vertex_star.get_tile(&atlas, &Point(1.5, 3_f64.sqrt() / 2.)).unwrap();
        assert_eq!(6, tile.size());
        assert_eq!(Point(2., 0.), tile.centroid);

        let tile = vertex_star.get_tile(&atlas, &Point(1.5, -3_f64.sqrt() / 2.)).unwrap();
        assert_eq!(6, tile.size());
        assert_eq!(Point(0.5, -3_f64.sqrt() / 2.), tile.centroid);
    }

    #[test]
    // 6*π/6  aka _6apio6: https://www.desmos.com/calculator/rstmycplwn
    // 6**π/2 aka _6aapio2: https://www.desmos.com/calculator/d62rvqroz4
    fn test_vertex_star_get_tile_4_6apio6_6aapio2_6apio6() {
        let num_rotations = 8;
        for degrees in (0..num_rotations).map(|i| (i as f64) * 360. / (num_rotations as f64)) {
          println!("degrees: {}", fmt_float(degrees, 2));

          let atlas = get_test_atlas_4_6apio6_6aapio2_6apio6();

          let x = X.transform(&Euclid::Rotate(to_rad(degrees)));
          let y = Y.transform(&Euclid::Rotate(to_rad(degrees)));

          // 2π * (n - 2) / 4n = angle of inclination to n-gon centroid (when first edge extends from origin to another point on the x-axis)
          // In this case we want to find the centroid angle of the circumscribing hexagon so we use n = 6.
          let hex_centroid_angle = to_rad(360. * (6. - 2.) / (4. * 6.));

          let _4_radius = 2_f64.sqrt() / 2.;

          let _6apio6_dent_angle = to_rad(45.);
          let _6aapio2_dent_angle = to_rad(15.);

          let _6apio6_outer_radius = _6apio6_dent_angle.cos() / hex_centroid_angle.cos();
          let _6aapio2_outer_radius = _6aapio2_dent_angle.cos() / hex_centroid_angle.cos();

          let _6apio6_inner_radius = _6apio6_dent_angle.cos() * hex_centroid_angle.tan() - _6apio6_dent_angle.sin();
          let _6aapio2_inner_radius = _6aapio2_dent_angle.cos() * hex_centroid_angle.tan() - _6aapio2_dent_angle.sin();

          // vertex star 0
          let vertex_star = VertexStar::new(&atlas, ORIGIN, 0, false, to_rad(degrees));

          // 6*π/6
          let tile = vertex_star.get_tile(&atlas, &x).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().proto_tiles.get(1).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _6apio6_inner_radius * to_rad(225. + degrees).cos(),
                  _6apio6_inner_radius * to_rad(225. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 6**π/2
          let tile = vertex_star.get_tile(&atlas, &y).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().proto_tiles.get(0).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _6aapio2_outer_radius * (hex_centroid_angle + to_rad(-15. + degrees)).cos(),
                  _6aapio2_outer_radius * (hex_centroid_angle + to_rad(-15. + degrees)).sin(),
              ),
              tile.centroid,
          );

          // vertex star 1
          let vertex_star = VertexStar::new(&atlas, ORIGIN, 1, false, to_rad(degrees));

          // 6**π/2
          let tile = vertex_star.get_tile(&atlas, &x).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().proto_tiles.get(3).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _6aapio2_inner_radius * to_rad(270. - 15. + degrees).cos(),
                  _6aapio2_inner_radius * to_rad(270. - 15. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 6*π/6
          let tile = vertex_star.get_tile(&atlas, &x.transform(&Euclid::Rotate(to_rad(30.)))).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().proto_tiles.get(0).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _6apio6_outer_radius * to_rad(15. + degrees).cos(),
                  _6apio6_outer_radius * to_rad(15. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 4
          let tile = vertex_star.get_tile(&atlas, &x.transform(&Euclid::Rotate(to_rad(30. + 90.)))).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().proto_tiles.get(1).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _4_radius * to_rad(30. + 45. + degrees).cos(),
                  _4_radius * to_rad(30. + 45. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 6*π/6
          let tile = vertex_star.get_tile(&atlas, &x.transform(&Euclid::Rotate(to_rad(30. + 90. + 30.)))).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().proto_tiles.get(2).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _6apio6_outer_radius * to_rad(30. + 90. + 15. + degrees).cos(),
                  _6apio6_outer_radius * to_rad(30. + 90. + 15. + degrees).sin(),
              ),
              tile.centroid,
          );

          // vertex star 2
          let vertex_star = VertexStar::new(&atlas, ORIGIN, 2, false, to_rad(degrees));

          // 6*π/6
          let tile = vertex_star.get_tile(&atlas, &x).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().proto_tiles.get(1).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _6apio6_inner_radius * to_rad(225. + degrees).cos(),
                  _6apio6_inner_radius * to_rad(225. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 4
          let tile = vertex_star.get_tile(&atlas, &y).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().proto_tiles.get(0).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _4_radius * to_rad(45. + degrees).cos(),
                  _4_radius * to_rad(45. + degrees).sin(),
              ),
              tile.centroid,
          );
        }
      }

    #[test]
    fn test_vertex_star_mutual_parity() {
        let atlas = get_test_atlas_4_4_4_4();

        let vertex_star = VertexStar::new(&atlas, Point(0.,0.), 0, false, 0.);
        assert_eq!(false, vertex_star.mutual_parity(false));
        assert_eq!(true, vertex_star.mutual_parity(true));

        let vertex_star = VertexStar::new(&atlas, Point(0.,0.), 0, true, 0.);
        assert_eq!(true, vertex_star.mutual_parity(false));
        assert_eq!(false, vertex_star.mutual_parity(true));
    }

    #[test]
    fn test_patch_insert_tile_by_point_1() {
        let mut patch = Patch::new(
            get_test_atlas_4_4_4_4(),
            get_tile_tree_config(),
            get_vertex_star_tree_config(),
        ).expect("");

        patch.insert_tile_by_point(Point(-2.3666666666666667, 1.729999796549479)).unwrap();
    }

    #[test]
    fn test_patch_insert_tile_by_point_2() {
        let mut patch = Patch::new(
            get_test_atlas_4_4_4_4(),
            get_tile_tree_config(),
            get_vertex_star_tree_config(),
        ).expect("");

        patch.insert_tile_by_point(Point(-3.966666666666667, 5.729999796549479)).unwrap();
    }

    #[test]
    fn test_patch_insert_tile_by_point_3() {
        let mut patch = Patch::new(
            get_test_atlas_4_4_4_4(),
            get_tile_tree_config(),
            get_vertex_star_tree_config(),
        ).expect("");

        patch.insert_tile_by_point(Point(-3.9, 2.296666463216146)).unwrap();
    }

    #[test]
    fn test_patch_insert_tile_by_point_4() {
        let mut patch = Patch::new(
            get_test_atlas_6_6_6(),
            get_tile_tree_config(),
            get_vertex_star_tree_config(),
        ).expect("");

        patch.insert_tile_by_point(Point(1.2333333333333334, 5.729999796549479)).unwrap();
    }

    #[test]
    fn test_patch_insert_tile_by_point_5() {
        let mut patch = Patch::new(
            get_test_atlas_6_6_6(),
            get_tile_tree_config(),
            get_vertex_star_tree_config(),
        ).expect("");

        patch.insert_tile_by_point(Point(1.3666666666666667, 5.696666463216146)).unwrap();
    }

    #[test]
    fn test_patch_insert_tile_by_point_6() {
        let mut patch = Patch::new(
            get_test_atlas_6_6_6(),
            get_tile_tree_config(),
            get_vertex_star_tree_config(),
        ).expect("");

        patch.insert_tile_by_point(Point(4.600000000000001, 7.396666463216146)).unwrap();
    }
}
