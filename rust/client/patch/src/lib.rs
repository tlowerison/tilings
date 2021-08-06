use atlas::{Atlas, ProtoNeighbor, ProtoVertexStar};
use common::*;
use geometry::{Affine, Bounds, Euclid, Point, Spatial, Transform, Transformable};
use itertools::izip;
use pmr_quad_tree::{Config as TreeConfig, RcItem, Tree, WeakItem};
use std::{
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
        let mut tile_index = match self.link_map.get(neighbor_point) { None => return None, Some(i) => *i };
        if !self.parity {
            tile_index = (tile_index + self.size() - 1) % self.size();
        }
        let tile = match proto_vertex_star.tiles.get(tile_index) { None => return None, Some(pt) => pt };
        let reference_frame = VertexStar::reference_frame(self.parity, self.rotation);
        let mut tile = tile.transform(&reference_frame.transform(&Euclid::Translate(self.point.values())));
        tile.parity = self.parity;
        Some(tile)
    }

    // get_tile_centroid mirrors get_tile but only computes the tile's centroid point
    pub fn get_tile_centroid(&self, atlas: &Atlas, neighbor_point: &Point) -> Option<Point> {
        let proto_vertex_star = match self.get_proto_vertex_star(atlas) { None => return None, Some(pvs) => pvs };
        let mut tile_index = match self.link_map.get(neighbor_point) { None => return None, Some(i) => *i };
        if !self.parity {
            tile_index = (tile_index + self.size() - 1) % self.size();
        }
        let tile = match proto_vertex_star.tiles.get(tile_index) { None => return None, Some(pt) => pt };
        let reference_frame = VertexStar::reference_frame(self.parity, self.rotation);
        Some(tile.centroid.transform(&reference_frame.transform(&Euclid::Translate(self.point.values()))))
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
pub struct PatchTile<State> {
    pub tile: Tile,
    pub state: Option<State>, // if state is None, tile isn't drawn
}

impl<State> std::ops::Deref for PatchTile<State> {
    type Target = Tile;
    fn deref(&self) -> &Tile {
        &self.tile
    }
}

impl<State> Spatial for PatchTile<State> {
    type Hashed = Point;
    fn distance(&self, point: &Point) -> f64 { self.tile.distance(point) }
    fn intersects(&self, bounds: &Bounds) -> bool { self.tile.intersects(bounds) }
    fn key(&self) -> Self::Hashed { self.tile.key() }
}

#[derive(Debug)]
pub struct Patch<State> {
    pub atlas: Atlas,
    pub tile_diffs: HashMap<Point, (WeakItem<Point, PatchTile<State>>, TileDiff)>,
    pub vertex_stars: Tree<Point, VertexStar>,
    pub patch_tiles: Tree<Point, PatchTile<State>>,
}

impl<State> Patch<State> {
    // new creates a new Patch and inserts a single VertexStar and its first Tile
    pub fn new(atlas: Atlas, tile_tree_config: TreeConfig, vertex_star_tree_config: TreeConfig) -> Result<Patch<State>, String> {
        let mut vertex_stars: Tree<Point, VertexStar> = Tree::new(vertex_star_tree_config, false);
        vertex_stars.insert(VertexStar::new(&atlas, Point(0., 0.), 0, false, 0.));

        let patch_tiles = Tree::new(tile_tree_config, true);

        Ok(Patch {
            atlas,
            vertex_stars,
            patch_tiles,
            tile_diffs: HashMap::default(),
        })
    }

    pub fn drain_tile_diffs(&mut self) -> Vec<(RcItem<Point, PatchTile<State>>, TileDiff)> {
        self.tile_diffs
            .drain()
            .filter_map(|(_, (patch_tile_weak_item, tile_diff))| patch_tile_weak_item
                    .upgrade()
                    .map(|patch_tile_rc_item| (patch_tile_rc_item, tile_diff))
            )
            .collect()
    }

    pub fn insert_tile_by_point(&mut self, point: Point, state: Option<State>) -> Result<(), String> {
        let mut nearest_vertex_star = self.vertex_stars
            .nearest_neighbor(&point)
            .map_err(|e| format!("no nearby vertex stars:\n{}\n{:?}\n{:#?}", e, point, self.vertex_stars))?
            .item
            .upgrade()
            .ok_or("vertex star doesn't exist")?;

        let mut count = 0;
        loop {
            if count == 100 {
                return Err(format!("unable to add tile - too far"));
            }
            let next_vertex_star = nearest_vertex_star
                .value()
                .nearest_neighbor(&self.atlas, &point)
                .ok_or("failed to find nearest vertex star")?;

            let tile = nearest_vertex_star
                .value()
                .get_tile(&self.atlas, &next_vertex_star.point)
                .ok_or("couldn't get new tile")?;

            let edge = (nearest_vertex_star.value().point.clone(), next_vertex_star.point.clone());

            if tile.contains(&point) {
                return self.insert_adjacent_tile_by_edge(edge, state);
            } else {
                self.insert_adjacent_tile_by_edge(edge, None)?;
                self.vertex_stars.insert(next_vertex_star);
                nearest_vertex_star = self.vertex_stars
                    .nearest_neighbor(&point)
                    .map_err(|e| format!("no nearby vertex stars:\n{}\n{:?}\n{:#?}", e, point, self.vertex_stars))?
                    .item
                    .upgrade()
                    .ok_or("vertex star doesn't exist")?;
            }
            count += 1;
        }
    }

    pub fn get_tile_neighbor_centroids(&self, point: &Point) -> Option<Vec<Point>> {
        let nearest_patch_tile_rc = match self.patch_tiles.nearest_neighbor(&point).ok() { Some(a) => a, _ => return None };
        let nearest_patch_tile_rc = match nearest_patch_tile_rc.item.upgrade() { Some(a) => a, _ => return None };
        let nearest_patch_tile = nearest_patch_tile_rc.value();

        if !nearest_patch_tile.tile.contains(point) {
            return None
        }
        if let Some(patch_tile_rc) = self.patch_tiles.get(&nearest_patch_tile.tile.centroid) {
            return Some(
                patch_tile_rc
                    .neighbors()
                    .iter()
                    .filter_map(|(centroid, patch_tile_weak)| {
                        let patch_tile_rc = match patch_tile_weak.upgrade() { Some(a) => a, _ => return None };
                        let x = match &patch_tile_rc.value().state {
                            Some(_) => Some(centroid.clone()),
                            _ => None,
                        }; x
                    })
                    .collect()
            )
        }
        None
    }

    // insert_adjacent_tile_by_edge inserts a new Tile into this Patch
    // given a particular edge along which the Tile shares. In order to succeed,
    // both points in the edge are expected to be points of existing VertexStars
    // in this Patch. If both exist, the new Tile will be added starboard of the
    // edge drawn from start to stop.
    fn insert_adjacent_tile_by_edge(&mut self, (start, stop): (Point, Point), state: Option<State>) -> Result<(), String> {
        let start_vertex_star_rc = self.vertex_stars.get(&start)
            .ok_or_else(|| String::from(format!("no VertexStar found at start {}", start)))?;
        let start_vertex_star = start_vertex_star_rc.value();
        let tile = start_vertex_star.get_tile(&self.atlas, &stop).ok_or_else(|| String::from(format!("stop {} is not in the link of start {}", stop, start)))?;

        // store info we need after move
        let centroid = tile.centroid.clone();
        let tile_size = tile.size();
        let included = if let Some(_) = &state { true } else { false };

        if let Some(patch_tile_item) = self.patch_tiles.get(&centroid) {
            if included {
                if let None = &patch_tile_item.value().state {
                    self.patch_tiles.insert(PatchTile { tile, state });
                    self.insert_in_tile_diffs(centroid, TileDiff::Added)?;
                }
            }
            return Ok(())
        }

        self.patch_tiles.insert(PatchTile { tile, state });

        let mut link_points: Vec<(usize, Point)> = vec![(0, stop.clone()), (0, start.clone())];
        let mut new_link_points: Vec<(usize, Point)> = vec![];
        let mut reverse = stop.clone();
        let mut middle = start.clone();

        for _ in 0 .. tile_size - 1 {
            let middle_vertex_star_rc = self.vertex_stars.get(&middle)
                .ok_or_else(|| String::from(format!("missing VertexStar at {}", middle)))?;
            let middle_vertex_star = middle_vertex_star_rc.value();
            let forward_index = middle_vertex_star.get_clockwise_adjacent_link_index(&reverse).ok_or_else(|| String::from(format!("no link point found clockwise adjacent of {} for VertexStar {}", reverse, middle)))?;
            let forward = middle_vertex_star.link_vec.get(forward_index).map(|p| p.clone()).ok_or_else(|| String::from(format!("out of bounds index {} in VertexStar {}", forward_index, middle)))?;
            link_points.push((forward_index, forward));
            if let Some(forward_vertex_star_rc) = self.vertex_stars.get(&forward) {
                reverse = middle;
                middle = forward_vertex_star_rc.value().point.clone();
            } else {
                let vs = middle_vertex_star.get_neighbor_vertex_star(&self.atlas, forward_index)
                    .ok_or_else(|| String::from(format!("unable to create neighbor VertexStar of VertexStar {} for neighbor index {} at point {}", middle, forward_index, forward)))?;
                reverse = middle;
                middle = vs.point.clone();
                if !self.vertex_stars.has(&forward) {
                    new_link_points.push((forward_index, forward));
                    self.vertex_stars.insert(vs);
                }
            }
        }

        if included {
            self.insert_in_tile_diffs(centroid, TileDiff::Added)?;
        }

        Ok(())
    }

    fn insert_in_tile_diffs(&mut self, centroid: Point, tile_diff: TileDiff) -> Result<(), String> {
        let patch_tile_item = self.patch_tiles.get(&centroid).ok_or_else(|| "failed to get patch tile properly")?;
        self.update_neighbors_after_new_tile_insert(&centroid).or_else(|_| Err(String::from("couldn't update neighbors")))?;
        self.tile_diffs.insert(centroid, (patch_tile_item.downgrade(), tile_diff));
        Ok(())
    }

    fn update_neighbors_after_new_tile_insert(&mut self, tile_centroid: &Point) -> Result<(), ()> {
        let mut rc_item = self.patch_tiles.get(tile_centroid).ok_or_else(|| ())?;

        let neighbor_centroids: Vec<Result<Point, String>> = {
            let value = rc_item.value();
            Point::edges(&value.tile.points)
                .into_iter()
                .map(|edge| {
                    if let Some(vertex_star_rc) = self.vertex_stars.get(edge.0) {
                        let vertex_star = vertex_star_rc.value();
                        if let Some(centroid) = vertex_star.get_tile_centroid(&self.atlas, edge.1) {
                            return Ok(centroid)
                        } else {
                            return Err(String::from("b"))
                        }
                    } else {
                        return Err(format!("{}", edge.0))
                    }
                })
                .collect()
        };

        let mut item_neighbors = rc_item.neighbors_mut().map_err(|_| ())?;

        for centroid in neighbor_centroids.into_iter() {
            if let Ok(centroid) = centroid {
                if let Some(mut neighbor_rc_item) = self.patch_tiles.get(&centroid) {
                    item_neighbors.insert(centroid, neighbor_rc_item.downgrade());

                    let rc_item = self.patch_tiles.get(tile_centroid).ok_or_else(|| ())?;
                    let mut neighbor_item_neighbors = neighbor_rc_item.neighbors_mut().map_err(|_| ())?;
                    neighbor_item_neighbors.insert(rc_item.value().tile.centroid.clone(), rc_item.downgrade());
                }
            } else {
                return Err(())
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas::VertexStarTransform;
    use tile::Tile;
    use geometry::Point;
    use std::f64::consts::{PI, TAU};

    const ORIGIN: Point = Point(0., 0.);
    const X: Point = Point(1., 0.);
    const Y: Point = Point(0., 1.);

    #[derive(Debug)]
    pub struct TestTile(pub Tile);

    impl TestTile {
        pub fn new(tile: &Tile) -> TestTile {
            TestTile(tile.clone())
        }
    }

    impl std::ops::Deref for TestTile {
        type Target = Tile;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Eq for TestTile {}

    impl PartialEq for TestTile {
        fn eq(&self, other: &Self) -> bool {
            if self.size() != other.size() {
                return false;
            }
            for (self_i, other_i) in izip!(
                rev_iter(self.0.parity, 0..self.size()),
                rev_iter(other.0.parity, 0..other.size())
            ) {
                if hash_float(self.angle(self_i), DEFAULT_PRECISION) != hash_float(other.angle(other_i), DEFAULT_PRECISION) {
                    return false;
                }
            }
            return true;
        }
    }

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
            tiles: vec![
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-1., 0.),
                        Point(-0.5, -0.8660254037844383),
                    ],
                    centroid: Point(-0.5, -0.2886751345948125),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(0.5, 0.8660254037844385),
                        Point(-0.5, 0.866025403784439),
                    ],
                    centroid: Point(0., 0.577350269189626),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-0.5, 0.866025403784439),
                        Point(-1., 0.),
                    ],
                    centroid: Point(-0.5, 0.2886751345948133),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-0.5, -0.8660254037844379),
                        Point(0.5, -0.8660254037844396),
                    ],
                    centroid: Point(-0., -0.577350269189626),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(1., 0.),
                        Point(0.5, 0.8660254037844388),
                    ],
                    centroid: Point(0.5, 0.288675134594813),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(0.5, -0.8660254037844395),
                        Point(1., -0.),
                    ],
                    centroid: Point(0.5, -0.28867513459481375),
                    parity: false,
                },
            ],
            proto_vertex_stars: vec![
                ProtoVertexStar {
                    index: 0,
                    tiles: vec![
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(1., 0.),
                                Point(0.5, 0.8660254037844388),
                            ],
                            centroid: Point(0.5, 0.288675134594813),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(0.5, 0.8660254037844385),
                                Point(-0.5, 0.866025403784439),
                            ],
                            centroid: Point(0., 0.577350269189626),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0.5, 0.866025403784439),
                                Point(-1., 0.),
                            ],
                            centroid: Point(-0.5, 0.2886751345948133),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-1., 0.),
                                Point(-0.5, -0.8660254037844383),
                            ],
                            centroid: Point(-0.5, -0.2886751345948125),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0.5, -0.8660254037844379),
                                Point(0.5, -0.8660254037844396),
                            ],
                            centroid: Point(-0., -0.577350269189626),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(0.5, -0.8660254037844395),
                                Point(1., -0.),
                            ],
                            centroid: Point(0.5, -0.28867513459481375),
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (1., 0.),
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
                                translate: (0.5, 0.8660254037844385),
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
                                translate: (-0.5, 0.866025403784439),
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
                                translate: (-1., 0.),
                                rotate: 0.,
                            },
                            neighbor_index: 0,
                            forward_tile_index: 0,
                            reverse_tile_index: 5,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (-0.5, -0.8660254037844379),
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
                                translate: (0.5, -0.8660254037844395),
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
            tiles: vec![
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-1., 0.),
                        Point(-1., -1.),
                        Point(-0., -1.),
                    ],
                    centroid: Point(-0.5, -0.5),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(0., 1.),
                        Point(-1., 1.),
                        Point(-1., 0.),
                    ],
                    centroid: Point(-0.5, 0.5),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(1., 0.),
                        Point(1., 1.),
                        Point(0., 1.),
                    ],
                    centroid: Point(0.5, 0.5),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-0., -1.),
                        Point(1., -1.),
                        Point(1., -0.),
                    ],
                    centroid: Point(0.5, -0.5),
                    parity: false,
                },
            ],
            proto_vertex_stars: vec![
                ProtoVertexStar {
                    index: 0,
                    tiles: vec![
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(1., 0.),
                                Point(1., 1.),
                                Point(0., 1.),
                            ],
                            centroid: Point(0.5, 0.5),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(0., 1.),
                                Point(-1., 1.),
                                Point(-1., 0.),
                            ],
                            centroid: Point(-0.5, 0.5),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-1., 0.),
                                Point(-1., -1.),
                                Point(-0., -1.),
                            ],
                            centroid: Point(-0.5, -0.5),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0., -1.),
                                Point(1., -1.),
                                Point(1., -0.),
                            ],
                            centroid: Point(0.5, -0.5),
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (1., 0.),
                                rotate: 3.141592653589793,
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
                                rotate: 4.71238898038469,
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
                                rotate: 0.,
                            },
                            neighbor_index: 0,
                            forward_tile_index: 0,
                            reverse_tile_index: 3,
                        },
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (-0., -1.),
                                rotate: 1.5707963267948966,
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
            tiles: vec![
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(1., 0.),
                        Point(1.5, 0.8660254037844386),
                        Point(1., 1.7320508075688774),
                        Point(0., 1.7320508075688776),
                        Point(-0.5, 0.8660254037844393),
                    ],
                    centroid: Point(0.5, 0.8660254037844389),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-0.5, -0.8660254037844397),
                        Point(0., -1.7320508075688772),
                        Point(1., -1.7320508075688754),
                        Point(1.5, -0.8660254037844358),
                        Point(1., 0.),
                    ],
                    centroid: Point(0.5, -0.8660254037844375),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-0.5, 0.866025403784438),
                        Point(-1.5, 0.8660254037844369),
                        Point(-2., -0.),
                        Point(-1.5, -0.8660254037844404),
                        Point(-0.5, -0.8660254037844397),
                    ],
                    centroid: Point(-1., -0.),
                    parity: false,
                },
            ],
            proto_vertex_stars: vec![
                ProtoVertexStar {
                    index: 0,
                    tiles: vec![
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(1., 0.),
                                Point(1.5, 0.8660254037844386),
                                Point(1., 1.7320508075688774),
                                Point(0., 1.7320508075688776),
                                Point(-0.5, 0.8660254037844393),
                            ],
                            centroid: Point(0.5, 0.8660254037844389),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0.5, 0.866025403784438),
                                Point(-1.5, 0.8660254037844369),
                                Point(-2., -0.),
                                Point(-1.5, -0.8660254037844404),
                                Point(-0.5, -0.8660254037844397),
                            ],
                            centroid: Point(-1., -0.),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0.5, -0.8660254037844397),
                                Point(0., -1.7320508075688772),
                                Point(1., -1.7320508075688754),
                                Point(1.5, -0.8660254037844358),
                                Point(1., 0.),
                            ],
                            centroid: Point(0.5, -0.8660254037844375),
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (1., 0.),
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
                                translate: (-0.5, 0.866025403784438),
                                rotate: 5.23598775598299,
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
                                rotate: 1.0471975511966,
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
            tiles: vec![
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(1., 0.),
                        Point(0.5, 0.8660254037844388),
                    ],
                    centroid: Point(0.5, 0.288675134594813),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(0.5, 0.8660254037844385),
                        Point(0.5, 1.8660254037844388),
                        Point(0., 2.732050807568877),
                        Point(-0.8660254037844373, 3.2320508075688776),
                        Point(-1.8660254037844375, 3.2320508075688785),
                        Point(-2.7320508075688763, 2.7320508075688785),
                        Point(-3.2320508075688763, 1.8660254037844404),
                        Point(-3.232050807568877, 0.8660254037844403),
                        Point(-2.7320508075688776, 0.),
                        Point(-1.8660254037844397, -0.5),
                        Point(-0.8660254037844396, -0.5),
                    ],
                    centroid: Point(-1.3660254037844384, 1.3660254037844393),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-0.8660254037844386, -0.5),
                        Point(-1.3660254037844386, -1.3660254037844388),
                        Point(-1.3660254037844386, -2.366025403784439),
                        Point(-0.8660254037844386, -3.232050807568877),
                        Point(0., -3.732050807568877),
                        Point(1., -3.7320508075688776),
                        Point(1.8660254037844386, -3.2320508075688776),
                        Point(2.366025403784439, -2.366025403784439),
                        Point(2.3660254037844393, -1.3660254037844388),
                        Point(1.8660254037844393, -0.5),
                        Point(1., -0.),
                    ],
                    centroid: Point(0.5, -1.8660254037844386),
                    parity: false,
                },
            ],
            proto_vertex_stars: vec![
                ProtoVertexStar {
                    index: 0,
                    tiles: vec![
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(1., 0.),
                                Point(0.5, 0.8660254037844388),
                            ],
                            centroid: Point(0.5, 0.288675134594813),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(0.5, 0.8660254037844385),
                                Point(0.5, 1.8660254037844388),
                                Point(0., 2.732050807568877),
                                Point(-0.8660254037844373, 3.2320508075688776),
                                Point(-1.8660254037844375, 3.2320508075688785),
                                Point(-2.7320508075688763, 2.7320508075688785),
                                Point(-3.2320508075688763, 1.8660254037844404),
                                Point(-3.232050807568877, 0.8660254037844403),
                                Point(-2.7320508075688776, 0.),
                                Point(-1.8660254037844397, -0.5),
                                Point(-0.8660254037844396, -0.5),
                            ],
                            centroid: Point(-1.3660254037844384, 1.3660254037844393),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0.8660254037844386, -0.5),
                                Point(-1.3660254037844386, -1.3660254037844388),
                                Point(-1.3660254037844386, -2.366025403784439),
                                Point(-0.8660254037844386, -3.232050807568877),
                                Point(0., -3.732050807568877),
                                Point(1., -3.7320508075688776),
                                Point(1.8660254037844386, -3.2320508075688776),
                                Point(2.366025403784439, -2.366025403784439),
                                Point(2.3660254037844393, -1.3660254037844388),
                                Point(1.8660254037844393, -0.5),
                                Point(1., -0.),
                            ],
                            centroid: Point(0.5, -1.8660254037844386),
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (1., 0.),
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
                                translate: (0.5, 0.8660254037844385),
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
                                translate: (-0.8660254037844386, -0.5),
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
            tiles: vec![
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-0.8660254037844389, 0.5),
                        Point(-1.7320508075688774, -0.),
                        Point(-1.7320508075688772, -1.),
                        Point(-0.8660254037844383, -1.5),
                        Point(0., -1.),
                    ],
                    centroid: Point(-0.8660254037844384, -0.5),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(1., 0.),
                        Point(1.8660254037844388, 0.5),
                        Point(2.366025403784439, 1.3660254037844386),
                        Point(2.366025403784439, 2.3660254037844384),
                        Point(1.866025403784439, 3.232050807568877),
                        Point(1., 3.732050807568877),
                        Point(0., 3.732050807568877),
                        Point(-0.8660254037844384, 3.2320508075688776),
                        Point(-1.3660254037844388, 2.3660254037844393),
                        Point(-1.366025403784439, 1.3660254037844393),
                        Point(-0.866025403784439, 0.5),
                    ],
                    centroid: Point(0.5, 1.8660254037844388),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(0., -1.),
                        Point(1., -1.),
                        Point(1., 0.),
                    ],
                    centroid: Point(0.5, -0.5),
                    parity: false,
                },
            ],
            proto_vertex_stars: vec![
                ProtoVertexStar {
                    index: 0,
                    tiles: vec![
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(1., 0.),
                                Point(1.8660254037844388, 0.5),
                                Point(2.366025403784439, 1.3660254037844386),
                                Point(2.366025403784439, 2.3660254037844384),
                                Point(1.866025403784439, 3.232050807568877),
                                Point(1., 3.732050807568877),
                                Point(0., 3.732050807568877),
                                Point(-0.8660254037844384, 3.2320508075688776),
                                Point(-1.3660254037844388, 2.3660254037844393),
                                Point(-1.366025403784439, 1.3660254037844393),
                                Point(-0.866025403784439, 0.5),
                            ],
                            centroid: Point(0.5, 1.8660254037844388),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0.8660254037844389, 0.5),
                                Point(-1.7320508075688774, -0.),
                                Point(-1.7320508075688772, -1.),
                                Point(-0.8660254037844383, -1.5),
                                Point(0., -1.),
                            ],
                            centroid: Point(-0.8660254037844384, -0.5),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(0., -1.),
                                Point(1., -1.),
                                Point(1., 0.),
                            ],
                            centroid: Point(0.5, -0.5),
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: true,
                                translate: (1., 0.),
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
                                translate: (-0.8660254037844389, 0.5),
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
                                translate: (0., -1.),
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
            tiles: vec![
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-0.5, 0.866025403784439),
                        Point(0.3660254037844396, 1.3660254037844384),
                        Point(-0.6339745962155605, 1.366025403784439),
                        Point(-0.63397459621556, 2.3660254037844393),
                        Point(-1.1339745962155607, 1.5),
                        Point(-2., 2.),
                        Point(-1.5, 1.133974596215562),
                        Point(-2.3660254037844384, 0.6339745962155628),
                        Point(-1.3660254037844386, 0.6339745962155625),
                        Point(-1.3660254037844388, -0.36602540378443754),
                        Point(-0.8660254037844389, 0.5),
                    ],
                    centroid: Point(-1., 1.),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(1., 0.),
                        Point(1., 1.),
                        Point(0., 1.),
                    ],
                    centroid: Point(0.5, 0.5),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-0.866025403784438, 0.5),
                        Point(-1.3660254037844395, -0.3660254037844366),
                        Point(-2.2320508075688785, -0.8660254037844354),
                        Point(-1.73205080756888, -1.7320508075688748),
                        Point(-1.7320508075688812, -2.732050807568875),
                        Point(-0.7320508075688813, -2.732050807568876),
                        Point(0.13397459621555682, -3.2320508075688767),
                        Point(0.633974596215558, -2.366025403784439),
                        Point(1.5, -1.8660254037844402),
                        Point(1., -1.),
                        Point(1., -0.),
                    ],
                    centroid: Point(-0.3660254037844413, -1.366025403784438),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(1., 0.),
                        Point(1.8660254037844388, -0.5),
                        Point(2.366025403784439, 0.3660254037844389),
                        Point(3.232050807568877, 0.866025403784439),
                        Point(2.732050807568877, 1.7320508075688776),
                        Point(2.7320508075688767, 2.7320508075688776),
                        Point(1.7320508075688767, 2.732050807568877),
                        Point(0.8660254037844378, 3.2320508075688767),
                        Point(0.36602540378443815, 2.366025403784438),
                        Point(-0.5, 1.8660254037844377),
                        Point(0., 1.),
                    ],
                    centroid: Point(1.366025403784439, 1.3660254037844386),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-0., 1.),
                        Point(-0.5, 0.13397459621556124),
                        Point(-1.3660254037844388, 0.633974596215561),
                        Point(-0.8660254037844388, -0.23205080756887755),
                        Point(-1.7320508075688772, -0.7320508075688779),
                        Point(-0.7320508075688771, -0.7320508075688779),
                        Point(-0.732050807568877, -1.7320508075688776),
                        Point(-0.23205080756887753, -0.8660254037844388),
                        Point(0.6339745962155615, -1.3660254037844384),
                        Point(0.13397459621556057, -0.5),
                        Point(1., 0.),
                    ],
                    centroid: Point(-0.3660254037844392, -0.36602540378443865),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(1., 0.),
                        Point(1., -1.),
                        Point(1.5, -0.1339745962155613),
                        Point(2.366025403784439, -0.6339745962155612),
                        Point(1.8660254037844388, 0.23205080756887744),
                        Point(2.732050807568877, 0.7320508075688776),
                        Point(1.7320508075688772, 0.7320508075688777),
                        Point(1.7320508075688772, 1.7320508075688776),
                        Point(1.2320508075688776, 0.8660254037844388),
                        Point(0.3660254037844387, 1.3660254037844384),
                        Point(0.8660254037844395, 0.5),
                    ],
                    centroid: Point(1.3660254037844393, 0.3660254037844386),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(0.8660254037844389, 0.5),
                        Point(0.3660254037844395, 1.3660254037844384),
                        Point(-0.5, 0.8660254037844389),
                    ],
                    centroid: Point(0.18301270189221974, 0.6830127018922192),
                    parity: false,
                },
            ],
            proto_vertex_stars: vec![
                ProtoVertexStar {
                    index: 0,
                    tiles: vec![
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(1., 0.),
                                Point(1.8660254037844388, -0.5),
                                Point(2.366025403784439, 0.3660254037844389),
                                Point(3.232050807568877, 0.866025403784439),
                                Point(2.732050807568877, 1.7320508075688776),
                                Point(2.7320508075688767, 2.7320508075688776),
                                Point(1.7320508075688767, 2.732050807568877),
                                Point(0.8660254037844378, 3.2320508075688767),
                                Point(0.36602540378443815, 2.366025403784438),
                                Point(-0.5, 1.8660254037844377),
                                Point(0., 1.),
                            ],
                            centroid: Point(1.366025403784439, 1.3660254037844386),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0., 1.),
                                Point(-0.5, 0.13397459621556124),
                                Point(-1.3660254037844388, 0.633974596215561),
                                Point(-0.8660254037844388, -0.23205080756887755),
                                Point(-1.7320508075688772, -0.7320508075688779),
                                Point(-0.7320508075688771, -0.7320508075688779),
                                Point(-0.732050807568877, -1.7320508075688776),
                                Point(-0.23205080756887753, -0.8660254037844388),
                                Point(0.6339745962155615, -1.3660254037844384),
                                Point(0.13397459621556057, -0.5),
                                Point(1., 0.),
                            ],
                            centroid: Point(-0.3660254037844392, -0.36602540378443865),
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 1,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (1., 0.),
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
                                translate: (-0., 1.),
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
                    tiles: vec![
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(1., 0.),
                                Point(1., -1.),
                                Point(1.5, -0.1339745962155613),
                                Point(2.366025403784439, -0.6339745962155612),
                                Point(1.8660254037844388, 0.23205080756887744),
                                Point(2.732050807568877, 0.7320508075688776),
                                Point(1.7320508075688772, 0.7320508075688777),
                                Point(1.7320508075688772, 1.7320508075688776),
                                Point(1.2320508075688776, 0.8660254037844388),
                                Point(0.3660254037844387, 1.3660254037844384),
                                Point(0.8660254037844395, 0.5),
                            ],
                            centroid: Point(1.3660254037844393, 0.3660254037844386),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(0.8660254037844389, 0.5),
                                Point(0.3660254037844395, 1.3660254037844384),
                                Point(-0.5, 0.8660254037844389),
                            ],
                            centroid: Point(0.18301270189221974, 0.6830127018922192),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0.5, 0.866025403784439),
                                Point(0.3660254037844396, 1.3660254037844384),
                                Point(-0.6339745962155605, 1.366025403784439),
                                Point(-0.63397459621556, 2.3660254037844393),
                                Point(-1.1339745962155607, 1.5),
                                Point(-2., 2.),
                                Point(-1.5, 1.133974596215562),
                                Point(-2.3660254037844384, 0.6339745962155628),
                                Point(-1.3660254037844386, 0.6339745962155625),
                                Point(-1.3660254037844388, -0.36602540378443754),
                                Point(-0.8660254037844389, 0.5),
                            ],
                            centroid: Point(-1., 1.),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0.866025403784438, 0.5),
                                Point(-1.3660254037844395, -0.3660254037844366),
                                Point(-2.2320508075688785, -0.8660254037844354),
                                Point(-1.73205080756888, -1.7320508075688748),
                                Point(-1.7320508075688812, -2.732050807568875),
                                Point(-0.7320508075688813, -2.732050807568876),
                                Point(0.13397459621555682, -3.2320508075688767),
                                Point(0.633974596215558, -2.366025403784439),
                                Point(1.5, -1.8660254037844402),
                                Point(1., -1.),
                                Point(1., -0.),
                            ],
                            centroid: Point(-0.3660254037844413, -1.366025403784438),
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (1., 0.),
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
                                translate: (0.8660254037844389, 0.5),
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
                                translate: (-0.5, 0.866025403784439),
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
                                translate: (-0.866025403784438, 0.5),
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
                    tiles: vec![
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(1., 0.),
                                Point(1., 1.),
                                Point(0., 1.),
                            ],
                            centroid: Point(0.5, 0.5),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0., 1.),
                                Point(-0.5, 0.13397459621556124),
                                Point(-1.3660254037844388, 0.633974596215561),
                                Point(-0.8660254037844388, -0.23205080756887755),
                                Point(-1.7320508075688772, -0.7320508075688779),
                                Point(-0.7320508075688771, -0.7320508075688779),
                                Point(-0.732050807568877, -1.7320508075688776),
                                Point(-0.23205080756887753, -0.8660254037844388),
                                Point(0.6339745962155615, -1.3660254037844384),
                                Point(0.13397459621556057, -0.5),
                                Point(1., 0.),
                            ],
                            centroid: Point(-0.3660254037844392, -0.36602540378443865),
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 1,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (1., 0.),
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
                                translate: (-0., 1.),
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
            tiles: vec![
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(0.8660254037844389, 0.5),
                        Point(0.8660254037844397, 1.5),
                        Point(0., 2.),
                        Point(-0.8660254037844376, 1.5),
                        Point(-0.8660254037844388, 0.5),
                    ],
                    centroid: Point(0., 1.),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-0.5, 0.8660254037844386),
                        Point(-0.5, -0.13397459621556135),
                        Point(-1.3660254037844388, -0.6339745962155614),
                        Point(-0.36602540378443876, -0.6339745962155613),
                        Point(0.13397459621556115, -1.5),
                        Point(0.13397459621556124, -0.5),
                        Point(1., 0.),
                    ],
                    centroid: Point(-0.18301270189221938, -0.31698729810778065),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-0.8660254037844389, 0.5),
                        Point(-0.8660254037844395, 1.5),
                        Point(-1.366025403784439, 0.6339745962155607),
                        Point(-2.3660254037844393, 0.6339745962155603),
                        Point(-1.5, 0.13397459621556074),
                        Point(-1.5, -0.8660254037844393),
                        Point(-1., -0.),
                    ],
                    centroid: Point(-1.1830127018922196, 0.3169872981077801),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(1., 0.),
                        Point(1.5, 0.8660254037844386),
                        Point(1., 1.7320508075688774),
                        Point(0., 1.7320508075688776),
                        Point(-0.5, 0.8660254037844393),
                    ],
                    centroid: Point(0.5, 0.8660254037844389),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(1., 0.),
                        Point(2., 0.),
                        Point(2.5, 0.8660254037844387),
                        Point(3., 1.7320508075688774),
                        Point(2.5, 2.598076211353316),
                        Point(2., 3.4641016151377544),
                        Point(1., 3.464101615137754),
                        Point(0., 3.4641016151377535),
                        Point(-0.5, 2.5980762113533147),
                        Point(-1., 1.732050807568876),
                        Point(-0.5, 0.8660254037844377),
                    ],
                    centroid: Point(1., 1.732050807568877),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(-1., 0.),
                        Point(-1.5, -0.8660254037844385),
                        Point(-2., -1.7320508075688772),
                        Point(-1.5, -2.598076211353316),
                        Point(-1., -3.4641016151377544),
                        Point(-0., -3.464101615137754),
                        Point(1., -3.4641016151377535),
                        Point(1.5, -2.5980762113533147),
                        Point(2., -1.7320508075688763),
                        Point(1.5, -0.8660254037844379),
                        Point(1., -0.),
                    ],
                    centroid: Point(-0., -1.732050807568877),
                    parity: false,
                },
                Tile {
                    points: vec![
                        Point(0., 0.),
                        Point(1., 0.),
                        Point(1.5, -0.8660254037844387),
                        Point(1.5, 0.1339745962155613),
                        Point(2.366025403784439, 0.6339745962155612),
                        Point(1.3660254037844388, 0.6339745962155613),
                        Point(0.866025403784439, 1.5),
                        Point(0.8660254037844388, 0.5),
                    ],
                    centroid: Point(1.1830127018922194, 0.31698729810778065),
                    parity: false,
                },
            ],
            proto_vertex_stars: vec![
                ProtoVertexStar {
                    index: 0,
                    tiles: vec![
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(1., 0.),
                                Point(1.5, -0.8660254037844387),
                                Point(1.5, 0.1339745962155613),
                                Point(2.366025403784439, 0.6339745962155612),
                                Point(1.3660254037844388, 0.6339745962155613),
                                Point(0.866025403784439, 1.5),
                                Point(0.8660254037844388, 0.5),
                            ],
                            centroid: Point(1.1830127018922194, 0.31698729810778065),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(0.8660254037844389, 0.5),
                                Point(0.8660254037844397, 1.5),
                                Point(0., 2.),
                                Point(-0.8660254037844376, 1.5),
                                Point(-0.8660254037844388, 0.5),
                            ],
                            centroid: Point(0., 1.),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0.8660254037844389, 0.5),
                                Point(-0.8660254037844395, 1.5),
                                Point(-1.366025403784439, 0.6339745962155607),
                                Point(-2.3660254037844393, 0.6339745962155603),
                                Point(-1.5, 0.13397459621556074),
                                Point(-1.5, -0.8660254037844393),
                                Point(-1., -0.),
                            ],
                            centroid: Point(-1.1830127018922196, 0.3169872981077801),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-1., 0.),
                                Point(-1.5, -0.8660254037844385),
                                Point(-2., -1.7320508075688772),
                                Point(-1.5, -2.598076211353316),
                                Point(-1., -3.4641016151377544),
                                Point(-0., -3.464101615137754),
                                Point(1., -3.4641016151377535),
                                Point(1.5, -2.5980762113533147),
                                Point(2., -1.7320508075688763),
                                Point(1.5, -0.8660254037844379),
                                Point(1., -0.),
                            ],
                            centroid: Point(-0., -1.732050807568877),
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 1,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (1., 0.),
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
                                translate: (0.8660254037844389, 0.5),
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
                                translate: (-0.8660254037844389, 0.5),
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
                                translate: (-1., 0.),
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
                    tiles: vec![
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(1., 0.),
                                Point(2., 0.),
                                Point(2.5, 0.8660254037844387),
                                Point(3., 1.7320508075688774),
                                Point(2.5, 2.598076211353316),
                                Point(2., 3.4641016151377544),
                                Point(1., 3.464101615137754),
                                Point(0., 3.4641016151377535),
                                Point(-0.5, 2.5980762113533147),
                                Point(-1., 1.732050807568876),
                                Point(-0.5, 0.8660254037844377),
                            ],
                            centroid: Point(1., 1.732050807568877),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0.5, 0.8660254037844386),
                                Point(-0.5, -0.13397459621556135),
                                Point(-1.3660254037844388, -0.6339745962155614),
                                Point(-0.36602540378443876, -0.6339745962155613),
                                Point(0.13397459621556115, -1.5),
                                Point(0.13397459621556124, -0.5),
                                Point(1., 0.),
                            ],
                            centroid: Point(-0.18301270189221938, -0.31698729810778065),
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (1., 0.),
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
                                translate: (-0.5, 0.8660254037844386),
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
                    tiles: vec![
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(1., 0.),
                                Point(1.5, 0.8660254037844386),
                                Point(1., 1.7320508075688774),
                                Point(0., 1.7320508075688776),
                                Point(-0.5, 0.8660254037844393),
                            ],
                            centroid: Point(0.5, 0.8660254037844389),
                            parity: false,
                        },
                        Tile {
                            points: vec![
                                Point(0., 0.),
                                Point(-0.5, 0.8660254037844377),
                                Point(-0.5, -0.13397459621556224),
                                Point(-1.3660254037844377, -0.6339745962155637),
                                Point(-0.3660254037844376, -0.633974596215562),
                                Point(0.13397459621556382, -1.5),
                                Point(0.13397459621556213, -0.5),
                                Point(1., 0.),
                            ],
                            centroid: Point(-0.1830127018922188, -0.316987298107781),
                            parity: false,
                        },
                    ],
                    proto_neighbors: vec![
                        ProtoNeighbor {
                            proto_vertex_star_index: 0,
                            transform: VertexStarTransform {
                                parity: false,
                                translate: (1., 0.),
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
                                translate: (-0.5, 0.8660254037844377),
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
        let _patch = match Patch::<()>::new(
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
          let exp_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().tiles.get(1).unwrap();
          assert_eq!(TestTile::new(exp_tile), TestTile::new(&Tile::new(tile.points.clone())));
          assert_eq!(
              Point(
                  _6apio6_inner_radius * to_rad(225. + degrees).cos(),
                  _6apio6_inner_radius * to_rad(225. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 6**π/2
          let tile = vertex_star.get_tile(&atlas, &y).unwrap();
          let exp_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().tiles.get(0).unwrap();
          assert_eq!(TestTile::new(exp_tile), TestTile::new(&Tile::new(tile.points.clone())));
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
          let exp_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().tiles.get(3).unwrap();
          assert_eq!(TestTile::new(exp_tile), TestTile::new(&Tile::new(tile.points.clone())));
          assert_eq!(
              Point(
                  _6aapio2_inner_radius * to_rad(270. - 15. + degrees).cos(),
                  _6aapio2_inner_radius * to_rad(270. - 15. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 6*π/6
          let tile = vertex_star.get_tile(&atlas, &x.transform(&Euclid::Rotate(to_rad(30.)))).unwrap();
          let exp_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().tiles.get(0).unwrap();
          assert_eq!(TestTile::new(exp_tile), TestTile::new(&Tile::new(tile.points.clone())));
          assert_eq!(
              Point(
                  _6apio6_outer_radius * to_rad(15. + degrees).cos(),
                  _6apio6_outer_radius * to_rad(15. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 4
          let tile = vertex_star.get_tile(&atlas, &x.transform(&Euclid::Rotate(to_rad(30. + 90.)))).unwrap();
          let exp_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().tiles.get(1).unwrap();
          assert_eq!(TestTile::new(exp_tile), TestTile::new(&Tile::new(tile.points.clone())));
          assert_eq!(
              Point(
                  _4_radius * to_rad(30. + 45. + degrees).cos(),
                  _4_radius * to_rad(30. + 45. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 6*π/6
          let tile = vertex_star.get_tile(&atlas, &x.transform(&Euclid::Rotate(to_rad(30. + 90. + 30.)))).unwrap();
          let exp_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().tiles.get(2).unwrap();
          assert_eq!(TestTile::new(exp_tile), TestTile::new(&Tile::new(tile.points.clone())));
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
          let exp_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().tiles.get(1).unwrap();
          assert_eq!(TestTile::new(exp_tile), TestTile::new(&Tile::new(tile.points.clone())));
          assert_eq!(
              Point(
                  _6apio6_inner_radius * to_rad(225. + degrees).cos(),
                  _6apio6_inner_radius * to_rad(225. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 4
          let tile = vertex_star.get_tile(&atlas, &y).unwrap();
          let exp_tile = vertex_star.get_proto_vertex_star(&atlas).unwrap().tiles.get(0).unwrap();
          assert_eq!(TestTile::new(exp_tile), TestTile::new(&Tile::new(tile.points.clone())));
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
        let mut patch = Patch::<()>::new(
            get_test_atlas_4_4_4_4(),
            get_tile_tree_config(),
            get_vertex_star_tree_config(),
        ).expect("");

        patch.insert_tile_by_point(Point(-2.3666666666666667, 1.729999796549479), None).unwrap();
    }

    #[test]
    fn test_patch_insert_tile_by_point_2() {
        let mut patch = Patch::<()>::new(
            get_test_atlas_4_4_4_4(),
            get_tile_tree_config(),
            get_vertex_star_tree_config(),
        ).expect("");

        patch.insert_tile_by_point(Point(-3.966666666666667, 5.729999796549479), None).unwrap();
    }

    #[test]
    fn test_patch_insert_tile_by_point_3() {
        let mut patch = Patch::<()>::new(
            get_test_atlas_4_4_4_4(),
            get_tile_tree_config(),
            get_vertex_star_tree_config(),
        ).expect("");

        patch.insert_tile_by_point(Point(-3.9, 2.296666463216146), None).unwrap();
    }

    #[test]
    fn test_patch_insert_tile_by_point_4() {
        let mut patch = Patch::<()>::new(
            get_test_atlas_6_6_6(),
            get_tile_tree_config(),
            get_vertex_star_tree_config(),
        ).expect("");

        patch.insert_tile_by_point(Point(1.2333333333333334, 5.729999796549479), None).unwrap();
    }

    #[test]
    fn test_patch_insert_tile_by_point_5() {
        let mut patch = Patch::<()>::new(
            get_test_atlas_6_6_6(),
            get_tile_tree_config(),
            get_vertex_star_tree_config(),
        ).expect("");

        patch.insert_tile_by_point(Point(1.3666666666666667, 5.696666463216146), None).unwrap();
    }

    #[test]
    fn test_patch_insert_tile_by_point_6() {
        let mut patch = Patch::<()>::new(
            get_test_atlas_6_6_6(),
            get_tile_tree_config(),
            get_vertex_star_tree_config(),
        ).expect("");

        patch.insert_tile_by_point(Point(4.600000000000001, 7.396666463216146), None).unwrap();
    }
}
