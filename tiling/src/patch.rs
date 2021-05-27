use crate::tiling::{ProtoNeighbor, ProtoVertexStar, Tiling};
use common::*;
use geometry::{Affine, Euclid, Point, Transform, Transformable};
use std::collections::HashMap;
use std::f64::consts::TAU;
use tile::Tile;

#[derive(Clone)]
pub struct VertexStar {
    pub proto_vertex_star_index: usize,
    pub point: Point,
    pub parity: bool, // whether the VertexStar's link is the cyclical reverse or not of the underlying ProtoVertexStar's link
    pub rotation: f64, // argument of VertexStar's first neighbor relative to the VertexStar's point
    pub link_vec: Vec<Point>, // points of VertexStar's neighboring VertexStars
    pub link_map: HashMap<Point, usize>, // each Point in the link maps to its index in link_vec
}

pub enum VertexStarErr {
    BadProtoRef,
    BadRef,
    ComponentOutOfBounds(usize, Point),
    ProtoVertexStarErr(String),
}

impl VertexStar {
    pub fn new(tiling: &Tiling, point: Point, proto_vertex_star_index: usize, parity: bool, rotation: f64) -> VertexStar {
        let proto_vertex_star = tiling.proto_vertex_stars.get(proto_vertex_star_index).unwrap();
        if proto_vertex_star_index == 0 {
            println!("{} {} {} {}", point, proto_vertex_star_index, parity, fmt_float(rotation / TAU * 360., 2));
            println!("{}", proto_vertex_star);
        }

        let mut link_vec: Vec<Point> = Vec::with_capacity(proto_vertex_star.size());

        let reference_frame = VertexStar::reference_frame(parity, rotation).transform(&Euclid::Translate(point.values()));

        let transform_link_point = |proto_neighbor: &ProtoNeighbor| Point::new(proto_neighbor.transform.translate).transform(&reference_frame);
        link_vec.extend(proto_vertex_star.proto_neighbors.iter().map(transform_link_point));

        let mut link_map: HashMap<Point, usize> = HashMap::with_capacity(proto_vertex_star.size());
        link_map.extend(link_vec.iter().enumerate().map(|(i, point)| (point.clone(), i)));

        VertexStar {
            point,
            proto_vertex_star_index,
            parity,
            rotation,
            link_vec,
            link_map,
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
    pub fn get_neighbor_vertex_star(&self, tiling: &Tiling, index: usize) -> Option<VertexStar> {
        let proto_vertex_star = self.get_proto_vertex_star(tiling).unwrap();
        let proto_neighbor = proto_vertex_star.proto_neighbors.get(index).unwrap();
        let neighbor_proto_vertex_star = tiling.proto_vertex_stars.get(proto_neighbor.proto_vertex_star_index).unwrap();

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
            tiling,
            &self.point + &neighbor_point_in_self_ref,
            proto_neighbor.proto_vertex_star_index,
            parity,
            rotation,
        ))
    }

    // get_proto_vertex_star optionally returns this VertexStar's referenced ProtoVertexStar given a tiling to index into.
    pub fn get_proto_vertex_star<'a>(&self, tiling: &'a Tiling) -> Option<&'a ProtoVertexStar> {
        tiling.proto_vertex_stars.get(self.proto_vertex_star_index)
    }

    // get_tile creates the Tile situated clockwise of the given point in this VertexStar's link
    pub fn get_tile(&self, tiling: &Tiling, neighbor_point: &Point) -> Option<Tile> {
        let proto_vertex_star = match self.get_proto_vertex_star(tiling) { None => return None, Some(pvs) => pvs };
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

    pub fn size(&self) -> usize {
        self.link_map.len()
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

pub enum PathErr {
    Missing(String),
    VertexStarErr(VertexStarErr),
}

impl Patch {
    // new creates a new Patch and inserts a single VertexStar and its first Tile
    pub fn new(tiling: Tiling) -> Result<(Patch, Point), String> {
        let mut vertex_stars: HashMap<Point, VertexStar> = HashMap::default();

        let vertex_star_0 = VertexStar::new(&tiling, Point(0., 0.), 0, false, 0.);
        let vertex_star_1 = vertex_star_0.get_neighbor_vertex_star(&tiling, 0).unwrap();
        let vertex_point_0 = vertex_star_0.point.clone();
        let vertex_point_1 = vertex_star_1.point.clone();

        vertex_stars.insert(vertex_point_0, vertex_star_0);
        vertex_stars.insert(vertex_point_1, vertex_star_1);

        let mut patch = Patch {
            tiling,
            vertex_stars,
            tile_diffs: HashMap::default(),
            tiles: HashMap::default(),
        };

        match patch.insert_adjacent_tile_by_edge((vertex_point_1, vertex_point_0)) {
            Ok(centroid) => Ok((patch, centroid)),
            Err(e) => Err(e),
        }
    }

    pub fn drain_tile_diffs(&mut self) -> HashMap<Tile, TileDiff> {
        self.tile_diffs.drain().collect()
    }

    pub fn drain_tiles(&mut self) -> HashMap<Tile, TileDiff> {
        self.tiles.drain().into_iter().map(|(_, tile)| (tile, TileDiff::Removed)).collect()
    }

    pub fn insert_adjacent_tile_by_point(&mut self, centroid: &Point, point: Point) -> Result<Point, String> {
        let tile = match self.tiles.get(centroid) { Some(t) => t, None => return Err(String::from(format!("no Tile in Patch centered at {}", centroid))) };
        let edge = tile.closest_edge(&point);
        let t = tile.clone();
        match self.insert_adjacent_tile_by_edge(edge) {
            Ok(new_centroid) => Ok(new_centroid),
            Err(e) => Err(String::from(format!("{}\n{}\n{}\n({},{})\n{}", centroid, point, t, edge.0, edge.1, e)))
        }
    }

    // insert_adjacent_tile_by_edge_index inserts a new Tile into this Patch
    // given a particular edge along which the Tile shares. In order to succeed,
    // both points in the edge are expected to be points of existing VertexStars
    // in this Patch. If both exist, the new Tile will be added starboard of the
    // edge drawn from start to stop.
    fn insert_adjacent_tile_by_edge(&mut self, (start, stop): (Point, Point)) -> Result<Point, String> {
        let start_vertex_star = match self.vertex_stars.get(&start) { Some(vs) => vs, None => return Err(String::from(format!("no VertexStar found at start {}\n{}", start, self))) };
        let tile = match start_vertex_star.get_tile(&self.tiling, &stop) { Some(t) => t, None => return Err(String::from(format!("stop {} is not in the link of start {}\n{}", stop, start, self))) };
        let tile_size = tile.size();

        match self.tiles.insert(tile.centroid.clone(), tile.clone()) { None => self.tile_diffs.insert(tile.clone(), TileDiff::Added), Some(_) => return Ok(tile.centroid.clone()) };

        let mut link_points: Vec<(usize, Point)> = vec![(0, stop.clone()), (0, start.clone())];
        let mut new_link_points: Vec<(usize, Point)> = vec![];
        let mut reverse = stop.clone();
        let mut middle = start.clone();

        for _ in 0 .. tile_size - 1 {
            let middle_vertex_star = match self.vertex_stars.get(&middle) { Some(vs) => vs, None => return Err(String::from(format!("missing VertexStar at {}\n{}", middle, self))) };
            let forward_index = match middle_vertex_star.get_clockwise_adjacent_link_index(&reverse) { Some(i) => i, None => return Err(String::from(format!("no link point found clockwise adjacent of {} for VertexStar {}\n{}", reverse, middle, self))) };
            let forward = match middle_vertex_star.link_vec.get(forward_index) { Some(p) => p.clone(), None => return Err(String::from(format!("out of bounds index {} in VertexStar {}\n{}", forward_index, middle, self))) };
            link_points.push((forward_index, forward));
            if let Some(vs) = self.vertex_stars.get(&forward) {
                reverse = middle;
                middle = vs.point.clone();
            } else {
                let vs = match middle_vertex_star.get_neighbor_vertex_star(&self.tiling, forward_index) { Some(vs) => vs, None => return Err(String::from(format!("unable to create neighbor VertexStar of VertexStar {} for neighbor index {} at point {}\n{}", middle, forward_index, forward, self))) };
                reverse = middle;
                middle = self.vertex_stars.entry(forward.clone()).or_insert({ new_link_points.push((forward_index, forward)); vs }).point.clone();
            }
        }
        Ok(tile.centroid.clone())
    }
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

impl std::fmt::Display for VertexStar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VertexStar:\n- point: {}\n- proto_vertex_star_index: {}\n- parity: {}\n- rotation: {}\n- link: {}",
            self.point,
            self.proto_vertex_star_index,
            self.parity,
            fmt_float(self.rotation, 3),
            self.link_vec.iter().map(|p| format!("{}", p)).collect::<Vec<String>>().join(", "),
        )
    }
}

impl std::fmt::Display for Patch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match writeln!(f, "Patch:") { Ok(_) => {}, Err(e) => return Err(e) }
        for vertex_star in self.vertex_stars.values() {
            match writeln!(f, "{}", vertex_star) { Ok(_) => {}, Err(e) => return Err(e) }
            match writeln!(f, "- components:") { Ok(_) => {}, Err(e) => return Err(e) }
        }
        match writeln!(f, "\n- tiles:") { Ok(_) => {}, Err(e) => return Err(e) }
        for (_, tile) in self.tiles.iter() {
            match writeln!(f, "  - {}", tile) { Ok(_) => {}, Err(e) => return Err(e) }
        }
        Ok(())
    }
}

impl std::fmt::Display for PathErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathErr::Missing(value) => write!(f, "PathErr: missing {}", value),
            PathErr::VertexStarErr(value) => write!(f, "PathErr: {}", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tile::{ProtoTile, regular_polygon, star_polygon};
    use tiling_config::{Component, Neighbor, Vertex};
    use geometry::Point;
    use std::f64::consts::{PI, TAU};

    const ORIGIN: Point = Point(0., 0.);
    const X: Point = Point(1., 0.);
    const Y: Point = Point(0., 1.);

    fn get_test_tiling_3_3_3_3_3_3() -> Tiling {
        let triangle = regular_polygon(1., 3);
        Tiling::new(
            tiling_config::Tiling(vec![Vertex {
                components: vec![
                    Component(triangle.clone(), 0),
                    Component(triangle.clone(), 0),
                    Component(triangle.clone(), 0),
                    Component(triangle.clone(), 0),
                    Component(triangle.clone(), 0),
                    Component(triangle.clone(), 0),
                ],
                neighbors: vec![
                    Neighbor(0, 3, false),
                    Neighbor(0, 4, false),
                    Neighbor(0, 5, false),
                    Neighbor(0, 0, false),
                    Neighbor(0, 1, false),
                    Neighbor(0, 2, false),
                ],
            }]),
        ).expect("couldn't create tiling")
    }

    fn get_test_tiling_4_4_4_4() -> Tiling {
        let square = regular_polygon(1., 4);
        Tiling::new(
            tiling_config::Tiling(vec![Vertex {
                components: vec![
                    Component(square.clone(), 0),
                    Component(square.clone(), 1),
                    Component(square.clone(), 2),
                    Component(square.clone(), 3),
                ],
                neighbors: vec![
                    Neighbor(0, 2, false),
                    Neighbor(0, 3, false),
                    Neighbor(0, 0, false),
                    Neighbor(0, 1, false),
                ],
            }]),
        ).expect("couldn't create tiling")
    }

    fn get_test_tiling_6_6_6() -> Tiling {
        let hexagon = regular_polygon(1., 6);
        Tiling::new(
            tiling_config::Tiling(vec![Vertex {
                components: vec![
                    Component(hexagon.clone(), 0),
                    Component(hexagon.clone(), 0),
                    Component(hexagon.clone(), 0),
                ],
                neighbors: vec![
                    Neighbor(0, 1, false),
                    Neighbor(0, 2, false),
                    Neighbor(0, 0, false),
                ],
            }]),
        ).expect("couldn't create tiling")
    }

    fn get_test_tiling_3_12_12() -> Tiling {
        let triangle = regular_polygon(1., 3);
        let dodecagon = regular_polygon(1., 12);
        Tiling::new(
            tiling_config::Tiling(vec![Vertex {
                components: vec![
                    Component(triangle.clone(), 0),
                    Component(dodecagon.clone(), 0),
                    Component(dodecagon.clone(), 0),
                ],
                neighbors: vec![
                    Neighbor(0, 1, false),
                    Neighbor(0, 0, false),
                    Neighbor(0, 2, false),
                ],
            }]),
        ).expect("couldn't create tiling")
    }

    fn get_test_tiling_4_6_12() -> Tiling {
        let square = regular_polygon(1., 4);
        let hexagon = regular_polygon(1., 6);
        let dodecagon = regular_polygon(1., 12);
        Tiling::new(
            tiling_config::Tiling(vec![Vertex {
                components: vec![
                    Component(dodecagon.clone(), 0),
                    Component(hexagon.clone(), 0),
                    Component(square.clone(), 0),
                ],
                neighbors: vec![
                    Neighbor(0, 0, true),
                    Neighbor(0, 1, true),
                    Neighbor(0, 2, true),
                ],
            }]),
        ).expect("couldn't create tiling")
    }

    fn get_test_tiling_4_6apio6_6aapio2_6apio6() -> Tiling {
        let square = regular_polygon(1., 4);
        let star_1 = star_polygon(1., 6, PI / 6.);
        let star_2 = star_polygon(1., 6, PI / 2.);

        Tiling::new(
            tiling_config::Tiling(vec![
                Vertex {
                    components: vec![
                        Component(star_2.clone(), 0),
                        Component(star_1.clone(), 1),
                    ],
                    neighbors: vec![
                        Neighbor(1, 0, false),
                        Neighbor(1, 3, false),
                    ],
                },
                Vertex {
                    components: vec![
                        Component(star_1.clone(), 0),
                        Component(square.clone(), 0),
                        Component(star_1.clone(), 0),
                        Component(star_2.clone(), 1),
                    ],
                    neighbors: vec![
                        Neighbor(0, 0, false),
                        Neighbor(2, 1, false),
                        Neighbor(2, 0, false),
                        Neighbor(0, 1, false),
                    ],
                },
                Vertex {
                    components: vec![
                        Component(square.clone(), 0),
                        Component(star_1.clone(), 1),
                    ],
                    neighbors: vec![
                        Neighbor(1, 2, false),
                        Neighbor(1, 1, false),
                    ],
                }
            ]),
        ).expect("couldn't create tiling")
    }

    #[test]
    fn test_tiling_6_4apio6_6_4apio6() {
        let star = star_polygon(1., 4, PI / 6.);
        let hexagon_1 = regular_polygon(1., 6);
        let hexagon_2 = star_polygon(1., 6, 2. * PI / 3.);

        let tiling = Tiling::new(
            tiling_config::Tiling(vec![
                Vertex {
                    components: vec![
                        Component(star.clone(), 0),
                        Component(hexagon_1.clone(), 0),
                        Component(star.clone(), 0),
                        Component(hexagon_2.clone(), 1),
                    ],
                    neighbors: vec![
                        Neighbor(1, 0, false),
                        Neighbor(2, 1, false),
                        Neighbor(2, 0, false),
                        Neighbor(1, 1, false),
                    ],
                },
                Vertex {
                    components: vec![
                        Component(hexagon_2.clone(), 0),
                        Component(star.clone(), 1),
                    ],
                    neighbors: vec![
                        Neighbor(0, 0, false),
                        Neighbor(0, 3, false),
                    ],
                },
                Vertex {
                    components: vec![
                        Component(hexagon_1.clone(), 0),
                        Component(star.clone(), 1),
                    ],
                    neighbors: vec![
                        Neighbor(0, 2, false),
                        Neighbor(0, 1, false),
                    ],
                },
            ]),
        ).expect("couldn't create tiling");

        let _patch = match Patch::new(tiling) {
            Ok(_) => {},
            Err(e) => {
                println!("{}", e);
                assert!(false);
            },
        };
    }

    // TODO: add 4_6apio6_6aapio2_6apio6 to the list of all test tilings
    fn get_all_test_tilings() -> [Tiling; 5] {
        [
            get_test_tiling_4_4_4_4(),
            get_test_tiling_3_3_3_3_3_3(),
            get_test_tiling_6_6_6(),
            get_test_tiling_3_12_12(),
            get_test_tiling_4_6_12(),
        ]
    }

    #[test]
    // by link refers to asserting that all of a vertex star's neighbors (i.e. the vertex star's link) are correctly configured
    fn test_vertex_star_get_neighbor_vertex_star_by_link() {
        let [
            tiling_4_4_4_4,
            tiling_3_3_3_3_3_3,
            tiling_6_6_6,
            tiling_3_12_12,
            tiling_4_6_12,
        ] = get_all_test_tilings();

        for rotation in (0..8).map(|i| rad((i as f64) * TAU / 8.)) {
            println!("rotation: {}", fmt_float(rotation / TAU * 360., 2));

            let rotate = Euclid::Rotate(rotation);

            let assert_vertex_star_neighbor = |
                tiling: &Tiling,
                vertex_star: &VertexStar,
                neighbor_index: usize,
                expected_point: Point,
                expected_parity: bool,
                expected_rotation: f64,
            | {
                println!("input: {} {} | expected: {} {} {}π", vertex_star.point, neighbor_index, expected_point.transform(&rotate), expected_parity, fmt_float(rad(expected_rotation + rotation) / PI, 2));
                let neighbor_vertex_star = vertex_star.get_neighbor_vertex_star(tiling, neighbor_index).unwrap();
                assert_eq!(expected_point.transform(&rotate), neighbor_vertex_star.point);
                assert_eq!(expected_parity, neighbor_vertex_star.parity);
                approx_eq!(f64, rad(expected_rotation + rotation), neighbor_vertex_star.rotation);
            };

            println!("Tiling 4.4.4.4");
            let vertex_star = VertexStar::new(&tiling_4_4_4_4, Point(0.,0.), 0, false, rotation);
            assert_vertex_star_neighbor(&tiling_4_4_4_4, &vertex_star, 0, X.transform(&Euclid::Rotate(to_rad(0. * 90.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_4_4_4_4, &vertex_star, 1, X.transform(&Euclid::Rotate(to_rad(1. * 90.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_4_4_4_4, &vertex_star, 2, X.transform(&Euclid::Rotate(to_rad(2. * 90.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_4_4_4_4, &vertex_star, 3, X.transform(&Euclid::Rotate(to_rad(3. * 90.))), false, 0.);
            println!();

            println!("Tiling 3.3.3.3.3.3");
            let vertex_star = VertexStar::new(&tiling_3_3_3_3_3_3, Point(0.,0.), 0, false, rotation);
            assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &vertex_star, 0, X.transform(&Euclid::Rotate(to_rad(0. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &vertex_star, 1, X.transform(&Euclid::Rotate(to_rad(1. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &vertex_star, 2, X.transform(&Euclid::Rotate(to_rad(2. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &vertex_star, 3, X.transform(&Euclid::Rotate(to_rad(3. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &vertex_star, 4, X.transform(&Euclid::Rotate(to_rad(4. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &vertex_star, 5, X.transform(&Euclid::Rotate(to_rad(5. * 60.))), false, 0.);
            println!();

            println!("Tiling 6.6.6");
            let vertex_star = VertexStar::new(&tiling_6_6_6, Point(0.,0.), 0, false, rotation);
            assert_vertex_star_neighbor(&tiling_6_6_6, &vertex_star, 0, X.transform(&Euclid::Rotate(to_rad(0. * 120.))), false, to_rad(60.));
            assert_vertex_star_neighbor(&tiling_6_6_6, &vertex_star, 1, X.transform(&Euclid::Rotate(to_rad(1. * 120.))), false, to_rad(60.));
            assert_vertex_star_neighbor(&tiling_6_6_6, &vertex_star, 2, X.transform(&Euclid::Rotate(to_rad(2. * 120.))), false, to_rad(60.));
            println!();

            println!("Tiling 3.12.12");
            let vertex_star = VertexStar::new(&tiling_3_12_12, Point(0.,0.), 0, false, rotation);
            assert_vertex_star_neighbor(&tiling_3_12_12, &vertex_star, 0, X, false, to_rad(120.));
            assert_vertex_star_neighbor(&tiling_3_12_12, &vertex_star, 1, X.transform(&Euclid::Rotate(to_rad(60.))), false, to_rad(240.));
            assert_vertex_star_neighbor(&tiling_3_12_12, &vertex_star, 2, X.transform(&Euclid::Rotate(to_rad(210.))), false, to_rad(180.));
            println!();

            println!("Tiling 4.6.12");
            let vertex_star = VertexStar::new(&tiling_4_6_12, Point(0.,0.), 0, false, rotation);
            assert_vertex_star_neighbor(&tiling_4_6_12, &vertex_star, 0, X, true, to_rad(180.));
            assert_vertex_star_neighbor(&tiling_4_6_12, &vertex_star, 1, X.transform(&Euclid::Rotate(to_rad(150.))), true, to_rad(120.));
            assert_vertex_star_neighbor(&tiling_4_6_12, &vertex_star, 2, X.transform(&Euclid::Rotate(to_rad(270.))), true, to_rad(0.));
            println!();
        }
    }

    #[test]
    // by chain refers to asserting that a sequence of vertex stars, the next accumulated as a neighbor of the previous star, are correctly configured
    fn test_vertex_star_get_neighbor_vertex_star_by_sequence() {
        let [
            tiling_4_4_4_4,
            tiling_3_3_3_3_3_3,
            tiling_6_6_6,
            tiling_3_12_12,
            tiling_4_6_12,
        ] = get_all_test_tilings();

        for rotation in (0..8).map(|i| rad((i as f64) * TAU / 8.)) {
            println!("rotation: {}", fmt_float(rotation / TAU * 360., 2));

            let rotate = Euclid::Rotate(rotation);

            let assert_vertex_star_neighbor = |
                tiling: &Tiling,
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

                let neighbor_vertex_star = vertex_star.get_neighbor_vertex_star(tiling, neighbor_index).unwrap();

                assert_eq!(expected_parity, neighbor_vertex_star.parity);
                assert_eq!(expected_point, neighbor_vertex_star.point);
                approx_eq!(f64, expected_rotation, neighbor_vertex_star.rotation);

                neighbor_vertex_star
            };

            println!("Tiling 4.4.4.4");
            let vertex_star = VertexStar::new(&tiling_4_4_4_4, Point(0.,0.), 0, false, rotation);
            let nvs = assert_vertex_star_neighbor(&tiling_4_4_4_4, &vertex_star, 0, X.transform(&Euclid::Rotate(to_rad(0. * 90.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&tiling_4_4_4_4, &nvs, 1, X.transform(&Euclid::Rotate(to_rad(1. * 90.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&tiling_4_4_4_4, &nvs, 2, X.transform(&Euclid::Rotate(to_rad(2. * 90.))), false, 0.);
            let _nvs = assert_vertex_star_neighbor(&tiling_4_4_4_4, &nvs, 3, X.transform(&Euclid::Rotate(to_rad(3. * 90.))), false, 0.);
            println!();

            println!("Tiling 3.3.3.3.3.3");
            let vertex_star = VertexStar::new(&tiling_3_3_3_3_3_3, Point(0.,0.), 0, false, rotation);
            let nvs = assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &vertex_star, 0, X.transform(&Euclid::Rotate(to_rad(0. * 60.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &nvs, 1, X.transform(&Euclid::Rotate(to_rad(1. * 60.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &nvs, 2, X.transform(&Euclid::Rotate(to_rad(2. * 60.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &nvs, 3, X.transform(&Euclid::Rotate(to_rad(3. * 60.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &nvs, 4, X.transform(&Euclid::Rotate(to_rad(4. * 60.))), false, 0.);
            let _nvs = assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &nvs, 5, X.transform(&Euclid::Rotate(to_rad(5. * 60.))), false, 0.);
            println!();

            println!("Tiling 6.6.6");
            let vertex_star = VertexStar::new(&tiling_6_6_6, Point(0.,0.), 0, false, rotation);
            let nvs = assert_vertex_star_neighbor(&tiling_6_6_6, &vertex_star, 0, X.transform(&Euclid::Rotate(to_rad(0. * 120.))), false, to_rad(60.));
            let nvs = assert_vertex_star_neighbor(&tiling_6_6_6, &nvs, 1, X.transform(&Euclid::Rotate(to_rad(1. * 120.))), false, to_rad(60.));
            let _nvs = assert_vertex_star_neighbor(&tiling_6_6_6, &nvs, 2, X.transform(&Euclid::Rotate(to_rad(2. * 120.))), false, to_rad(60.));
            println!();

            println!("Tiling 3.12.12");
            let vertex_star = VertexStar::new(&tiling_3_12_12, Point(0.,0.), 0, false, rotation);
            let nvs = assert_vertex_star_neighbor(&tiling_3_12_12, &vertex_star, 0, X, false, to_rad(120.));
            let nvs = assert_vertex_star_neighbor(&tiling_3_12_12, &nvs, 1, X.transform(&Euclid::Rotate(to_rad(60.))), false, to_rad(240.));
            let _nvs = assert_vertex_star_neighbor(&tiling_3_12_12, &nvs, 2, X.transform(&Euclid::Rotate(to_rad(210.))), false, to_rad(180.));
            println!();

            println!("Tiling 4.6.12");
            let vertex_star = VertexStar::new(&tiling_4_6_12, Point(0.,0.), 0, false, rotation);

            let nvs = vertex_star.get_neighbor_vertex_star(&tiling_4_6_12, 0).unwrap();
            assert_eq!(true, nvs.parity);
            assert_eq!(X.transform(&rotate), nvs.point);
            approx_eq!(f64, rad(to_rad(180.) + rotation), nvs.rotation);

            let nvs = nvs.get_neighbor_vertex_star(&tiling_4_6_12, 1).unwrap();
            assert_eq!(false, nvs.parity);
            assert_eq!((&X + &X.transform(&Euclid::Rotate(to_rad(30.)))).transform(&rotate), nvs.point);
            approx_eq!(f64, rad(to_rad(60.) + rotation), nvs.rotation);

            let nvs = nvs.get_neighbor_vertex_star(&tiling_4_6_12, 2).unwrap();
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
        let tiling = get_test_tiling_4_4_4_4();

        let proto_vertex_star_index = 0;
        let vertex_star = VertexStar::new(&tiling, Point(0.,0.), 0, false, 0.);
        let proto_vertex_star = vertex_star.get_proto_vertex_star(&tiling).unwrap();
        assert_eq!(proto_vertex_star.index, proto_vertex_star_index);
    }

    #[test]
    fn test_vertex_star_get_tile_4_6_12() {
        let tiling = get_test_tiling_4_6_12();

        let vertex_star = VertexStar::new(&tiling, ORIGIN.clone(), 0, false, 0.);

        let tile = vertex_star.get_tile(&tiling, &X).unwrap();
        assert_eq!(4, tile.size());
        assert_eq!(Point(0.5, -0.5), tile.centroid);

        let tile = vertex_star.get_tile(&tiling, &X.transform(&Euclid::Rotate(to_rad(150.)))).unwrap();
        assert_eq!(12, tile.size());
        assert_eq!(Point(0.5, 1. + 3_f64.sqrt() / 2.), tile.centroid);

        let tile = vertex_star.get_tile(&tiling, &(-Y)).unwrap();
        assert_eq!(6, tile.size());
        assert_eq!(Point(- 3_f64.sqrt() / 2., -0.5), tile.centroid);

        // flipped vertex star
        let vertex_star = VertexStar::new(&tiling, ORIGIN, 0, true, 0.);

        let tile = vertex_star.get_tile(&tiling, &X).unwrap();
        assert_eq!(12, tile.size());
        assert_eq!(Point(0.5, -(1. + 3_f64.sqrt() / 2.)), tile.centroid);

        let tile = vertex_star.get_tile(&tiling, &Y).unwrap();
        assert_eq!(4, tile.size());
        assert_eq!(Point(0.5, 0.5), tile.centroid);

        let tile = vertex_star.get_tile(&tiling, &X.transform(&Euclid::Rotate(to_rad(360. - 150.)))).unwrap();
        assert_eq!(6, tile.size());
        assert_eq!(Point(- 3_f64.sqrt() / 2., 0.5), tile.centroid);
    }

    #[test]
    fn test_vertex_star_get_tile_6_6_6() {
        let tiling = get_test_tiling_6_6_6();

        let vertex_star = VertexStar::new(&tiling, X, 0, false, to_rad(60.));

        let tile = vertex_star.get_tile(&tiling, &ORIGIN).unwrap();
        assert_eq!(6, tile.size());
        assert_eq!(Point(0.5, 3_f64.sqrt() / 2.), tile.centroid);

        let tile = vertex_star.get_tile(&tiling, &Point(1.5, 3_f64.sqrt() / 2.)).unwrap();
        assert_eq!(6, tile.size());
        assert_eq!(Point(2., 0.), tile.centroid);

        let tile = vertex_star.get_tile(&tiling, &Point(1.5, -3_f64.sqrt() / 2.)).unwrap();
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

          let tiling = get_test_tiling_4_6apio6_6aapio2_6apio6();

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
          let vertex_star = VertexStar::new(&tiling, ORIGIN, 0, false, to_rad(degrees));

          // 6*π/6
          let tile = vertex_star.get_tile(&tiling, &x).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&tiling).unwrap().proto_tiles.get(1).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _6apio6_inner_radius * to_rad(225. + degrees).cos(),
                  _6apio6_inner_radius * to_rad(225. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 6**π/2
          let tile = vertex_star.get_tile(&tiling, &y).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&tiling).unwrap().proto_tiles.get(0).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _6aapio2_outer_radius * (hex_centroid_angle + to_rad(-15. + degrees)).cos(),
                  _6aapio2_outer_radius * (hex_centroid_angle + to_rad(-15. + degrees)).sin(),
              ),
              tile.centroid,
          );

          // vertex star 1
          let vertex_star = VertexStar::new(&tiling, ORIGIN, 1, false, to_rad(degrees));

          // 6**π/2
          let tile = vertex_star.get_tile(&tiling, &x).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&tiling).unwrap().proto_tiles.get(3).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _6aapio2_inner_radius * to_rad(270. - 15. + degrees).cos(),
                  _6aapio2_inner_radius * to_rad(270. - 15. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 6*π/6
          let tile = vertex_star.get_tile(&tiling, &x.transform(&Euclid::Rotate(to_rad(30.)))).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&tiling).unwrap().proto_tiles.get(0).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _6apio6_outer_radius * to_rad(15. + degrees).cos(),
                  _6apio6_outer_radius * to_rad(15. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 4
          let tile = vertex_star.get_tile(&tiling, &x.transform(&Euclid::Rotate(to_rad(30. + 90.)))).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&tiling).unwrap().proto_tiles.get(1).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _4_radius * to_rad(30. + 45. + degrees).cos(),
                  _4_radius * to_rad(30. + 45. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 6*π/6
          let tile = vertex_star.get_tile(&tiling, &x.transform(&Euclid::Rotate(to_rad(30. + 90. + 30.)))).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&tiling).unwrap().proto_tiles.get(2).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _6apio6_outer_radius * to_rad(30. + 90. + 15. + degrees).cos(),
                  _6apio6_outer_radius * to_rad(30. + 90. + 15. + degrees).sin(),
              ),
              tile.centroid,
          );

          // vertex star 2
          let vertex_star = VertexStar::new(&tiling, ORIGIN, 2, false, to_rad(degrees));

          // 6*π/6
          let tile = vertex_star.get_tile(&tiling, &x).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&tiling).unwrap().proto_tiles.get(1).unwrap();
          assert_eq!(exp_proto_tile, &ProtoTile::new(tile.points.clone()));
          assert_eq!(
              Point(
                  _6apio6_inner_radius * to_rad(225. + degrees).cos(),
                  _6apio6_inner_radius * to_rad(225. + degrees).sin(),
              ),
              tile.centroid,
          );

          // 4
          let tile = vertex_star.get_tile(&tiling, &y).unwrap();
          let exp_proto_tile = vertex_star.get_proto_vertex_star(&tiling).unwrap().proto_tiles.get(0).unwrap();
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
        let tiling = get_test_tiling_4_4_4_4();

        let vertex_star = VertexStar::new(&tiling, Point(0.,0.), 0, false, 0.);
        assert_eq!(false, vertex_star.mutual_parity(false));
        assert_eq!(true, vertex_star.mutual_parity(true));

        let vertex_star = VertexStar::new(&tiling, Point(0.,0.), 0, true, 0.);
        assert_eq!(true, vertex_star.mutual_parity(false));
        assert_eq!(false, vertex_star.mutual_parity(true));
    }

    #[test]
    fn test_patch_insert_adjacent_tile_by_point() {
        let (mut patch, mut centroid) = Patch::new(get_test_tiling_4_6_12()).unwrap();
        centroid = match patch.insert_adjacent_tile_by_point(&centroid, Point(1.30, 1.30)) { Ok(p) => p, Err(e) => panic!("{}", e) };
        centroid = match patch.insert_adjacent_tile_by_point(&centroid, Point(6.53,-1.31)) { Ok(p) => p, Err(e) => panic!("{}", e) };
        centroid = match patch.insert_adjacent_tile_by_point(&centroid, Point(7.,-0.5)) { Ok(p) => p, Err(e) => panic!("{}", e) };
        match patch.insert_adjacent_tile_by_point(&centroid, Point(7.,-0.5)) { Ok(p) => p, Err(e) => panic!("{}", e) };

        let edge = (Point(1. + 3_f64.sqrt() / 2., 0.5), Point(1.5 + 3_f64.sqrt() / 2., 0.5 + 3_f64.sqrt() / 2.));

        let (mut patch, _centroid) = Patch::new(get_test_tiling_4_6_12()).unwrap();
        patch.insert_adjacent_tile_by_edge(edge).expect("");
    }

    #[test]
    fn test_patch_insert_adjacent_tile_by_edge_4_6apio6_6aapio2_6apio6() {
        let tiling = get_test_tiling_4_6apio6_6aapio2_6apio6();

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

        let (mut patch, mut centroid) = Patch::new(tiling).unwrap();

        assert_eq!(
            Point(
                _6aapio2_outer_radius * to_rad(45.).cos(),
                _6aapio2_outer_radius * to_rad(45.).sin(),
            ),
            centroid,
        );

        let tile = patch.tiles.get(&centroid).unwrap();

        let mut exp_proto_tile = patch.tiling.proto_vertex_stars.get(0).unwrap().proto_tiles.first().unwrap().clone();
        let mut proto_tile = ProtoTile::new(tile.points.clone());
        exp_proto_tile.reorient(&ORIGIN);
        proto_tile.reorient(&ORIGIN);
        assert_eq!(exp_proto_tile, proto_tile);

        // 6*π/6
        centroid = patch.insert_adjacent_tile_by_edge((ORIGIN.clone(), X.clone())).expect("");
        assert_eq!(
            Point(
                _6apio6_inner_radius * to_rad(225.).cos(),
                _6apio6_inner_radius * to_rad(225.).sin(),
            ),
            centroid,
        );

        let tile = patch.tiles.get(&centroid).unwrap();
        let mut exp_proto_tile = patch.tiling.proto_vertex_stars.get(0).unwrap().proto_tiles.get(1).unwrap().clone();
        let mut proto_tile = ProtoTile::new(tile.points.clone());
        exp_proto_tile.reorient(&ORIGIN);
        proto_tile.reorient(&ORIGIN);

        assert_eq!(exp_proto_tile, proto_tile);
        for (i, point) in proto_tile.points.iter().enumerate() {
            println!("{}: {}", i, point);
            patch.vertex_stars.get(point).unwrap();
        }

        // 6*π/6 - repeat
        centroid = patch.insert_adjacent_tile_by_edge((ORIGIN.clone(), X.clone())).expect("");
        assert_eq!(
            Point(
                _6apio6_inner_radius * to_rad(225.).cos(),
                _6apio6_inner_radius * to_rad(225.).sin(),
            ),
            centroid,
        );

        let tile = patch.tiles.get(&centroid).unwrap();
        let mut exp_proto_tile = patch.tiling.proto_vertex_stars.get(0).unwrap().proto_tiles.get(1).unwrap().clone();
        let mut proto_tile = ProtoTile::new(tile.points.clone());
        exp_proto_tile.reorient(&ORIGIN);
        proto_tile.reorient(&ORIGIN);

        assert_eq!(exp_proto_tile, proto_tile);
        for (i, point) in proto_tile.points.iter().enumerate() {
            println!("{}: {}", i, point);
            patch.vertex_stars.get(point).unwrap();
        }

        // 6**π/2
        centroid = patch.insert_adjacent_tile_by_edge((ORIGIN.clone(), Y.clone())).expect("");
        assert_eq!(
            Point(
                _6aapio2_outer_radius * to_rad(45.).cos(),
                _6aapio2_outer_radius * to_rad(45.).sin(),
            ),
            centroid,
        );

        let tile = patch.tiles.get(&centroid).unwrap();
        let mut exp_proto_tile = patch.tiling.proto_vertex_stars.get(0).unwrap().proto_tiles.get(0).unwrap().clone();
        let mut proto_tile = ProtoTile::new(tile.points.clone());
        exp_proto_tile.reorient(&ORIGIN);
        proto_tile.reorient(&ORIGIN);

        assert_eq!(exp_proto_tile, proto_tile);
        for (i, point) in proto_tile.points.iter().enumerate() {
            println!("{}: {}", i, point);
            patch.vertex_stars.get(point).unwrap();
        }

        // 6**π/2 - repeat (different edge)
        centroid = patch.insert_adjacent_tile_by_edge((X.clone(), ORIGIN.clone())).expect("");
        assert_eq!(
            Point(
                _6aapio2_outer_radius * to_rad(45.).cos(),
                _6aapio2_outer_radius * to_rad(45.).sin(),
            ),
            centroid,
        );

        let tile = patch.tiles.get(&centroid).unwrap();
        let mut exp_proto_tile = patch.tiling.proto_vertex_stars.get(0).unwrap().proto_tiles.get(0).unwrap().clone();
        let mut proto_tile = ProtoTile::new(tile.points.clone());
        exp_proto_tile.reorient(&ORIGIN);
        proto_tile.reorient(&ORIGIN);

        assert_eq!(exp_proto_tile, proto_tile);
        for (i, point) in proto_tile.points.iter().enumerate() {
            println!("{}: {}", i, point);
            patch.vertex_stars.get(point).unwrap();
        }

        // 6*π/6
        centroid = patch.insert_adjacent_tile_by_edge((
            X.clone(),
            patch.vertex_stars.get(&X).unwrap().link_vec.get(1).unwrap().clone(), // (0.13,-0.50)
        )).expect("");
        assert_eq!(
            Point(
                _6apio6_inner_radius * to_rad(225.).cos(),
                _6apio6_inner_radius * to_rad(225.).sin(),
            ),
            centroid,
        );

        let tile = patch.tiles.get(&centroid).unwrap();
        let mut exp_proto_tile = patch.tiling.proto_vertex_stars.get(0).unwrap().proto_tiles.get(1).unwrap().clone();
        let mut proto_tile = ProtoTile::new(tile.points.clone());
        exp_proto_tile.reorient(&ORIGIN);
        proto_tile.reorient(&ORIGIN);

        assert_eq!(exp_proto_tile, proto_tile);
        for (i, point) in proto_tile.points.iter().enumerate() {
            println!("{}: {}", i, point);
            patch.vertex_stars.get(point).unwrap();
        }

        // 4
        centroid = patch.insert_adjacent_tile_by_edge((
            X.clone(),
            patch.vertex_stars.get(&X).unwrap().link_vec.get(2).unwrap().clone(), // (1.50,-0.87)
        )).expect("");
        assert_eq!(
            &X + &Point(
                _4_radius * to_rad(255.).cos(),
                _4_radius * to_rad(255.).sin(),
            ),
            centroid,
        );

        let tile = patch.tiles.get(&centroid).unwrap();
        let mut exp_proto_tile = patch.tiling.proto_vertex_stars.get(1).unwrap().proto_tiles.get(1).unwrap().clone();
        let mut proto_tile = ProtoTile::new(tile.points.clone());
        exp_proto_tile.reorient(&ORIGIN);
        proto_tile.reorient(&ORIGIN);

        assert_eq!(exp_proto_tile, proto_tile);
        for (i, point) in proto_tile.points.iter().enumerate() {
            println!("{}: {}", i, point);
            patch.vertex_stars.get(point).unwrap();
        }

        // 6*π/6
        let start = X.mul(2. * _6aapio2_outer_radius).transform(&Euclid::Rotate(to_rad(45.))); // (2.73, 2.73)
        let stop = &start - &X; // (1.73, 2.73)
        let transform = Euclid::Translate((&start - &X).values());

        centroid = patch.insert_adjacent_tile_by_edge((start, stop)).expect("");
        assert_eq!(
            &start + &Point(
                _6apio6_inner_radius * to_rad(45.).cos(),
                _6apio6_inner_radius * to_rad(45.).sin(),
            ),
            centroid,
        );

        let tile = patch.tiles.get(&centroid).unwrap();
        let mut exp_proto_tile = patch.tiling.proto_vertex_stars.get(1).unwrap().proto_tiles.get(0).unwrap().clone().transform(&transform);
        let mut proto_tile = ProtoTile::new(tile.points.clone());
        exp_proto_tile.reorient(&start);
        proto_tile.reorient(&start);

        assert_eq!(exp_proto_tile, proto_tile);
        for (i, point) in proto_tile.points.iter().enumerate() {
            println!("{}: {}", i, point);
            patch.vertex_stars.get(point).unwrap();
        }

        // 4
        let start = &stop + &X.transform(&Euclid::Rotate(to_rad(30.))); // (2.60,3.23)
        let stop = stop; // (1.73, 2.73)
        centroid = patch.insert_adjacent_tile_by_edge((start, stop)).expect("");
        assert_eq!(
            &stop + &Point(
                _4_radius * to_rad(75.).cos(),
                _4_radius * to_rad(75.).sin(),
            ),
            centroid,
        );

        let tile = patch.tiles.get(&centroid).unwrap();
        let mut exp_proto_tile = patch.tiling.proto_vertex_stars.get(1).unwrap().proto_tiles.get(1).unwrap().clone().transform(&transform);
        let mut proto_tile = ProtoTile::new(tile.points.clone());
        exp_proto_tile.reorient(&start);
        proto_tile.reorient(&start);

        assert_eq!(exp_proto_tile, proto_tile);
        for (i, point) in proto_tile.points.iter().enumerate() {
            println!("{}: {}", i, point);
            patch.vertex_stars.get(point).unwrap();
        }
    }
}
