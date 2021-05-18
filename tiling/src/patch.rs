use crate::tile::Tile;
use crate::tiling::{ProtoVertexStar, Tiling};
use common::*;
use geometry::{Euclid, Point, Transformable};
use std::collections::HashMap;

#[derive(Clone)]
pub struct VertexStar {
    pub proto_vertex_star_index: usize,
    pub point: Point,
    pub parity: bool, // whether the VertexStar's link is ordered in the same order as its ProtoVertexStar's link i.e. [(1,0),(0,1),(-1,0),(0,-1)] or [(1,0),(0,-1),(-1,0),(0,1)]
    pub rotation: f64, // argument of VertexStar's first neighbor relative to the VertexStar's point
}

pub enum VertexStarErr {
    BadProtoRef,
    BadRef,
    ComponentOutOfBounds(usize, Point),
    ProtoVertexStarErr(String),
}

impl VertexStar {
    pub fn new(point: Point, proto_vertex_star_index: usize, parity: bool, rotation: f64) -> VertexStar {
        VertexStar {
            point,
            proto_vertex_star_index,
            parity,
            rotation,
        }
    }

    pub fn get_neighbor(&self, tiling: &Tiling, index: usize) -> Option<VertexStar> {
        let proto_vertex_star = self.get_proto_vertex_star(tiling).unwrap();
        let proto_neighbor = proto_vertex_star.proto_neighbors.get(index).unwrap();
        let neighbor_proto_vertex_star = tiling.proto_vertex_stars.get(proto_neighbor.proto_vertex_star_index).unwrap();

        let mut neighbor_point_in_self_ref = Point::new(proto_neighbor.transform.translate);
        if self.parity {
            neighbor_point_in_self_ref = neighbor_point_in_self_ref.transform(&Euclid::Flip(0.));
        }
        neighbor_point_in_self_ref = neighbor_point_in_self_ref.transform(&Euclid::Rotate(self.rotation));

        let neighbor_edge_point_in_neighbor_ref = Point::new(neighbor_proto_vertex_star.proto_neighbors.get(proto_neighbor.neighbor_index).unwrap().transform.translate);

        let neighbor_edge_rotation = rad((-neighbor_point_in_self_ref).arg() - neighbor_edge_point_in_neighbor_ref.arg());

        let mut neighbor_first_edge_point_transformed = Point::new(neighbor_proto_vertex_star.proto_neighbors.get(0).unwrap().transform.translate).transform(&Euclid::Rotate(neighbor_edge_rotation));

        let parity = self.mutual_parity(proto_neighbor.transform.parity);
        if parity {
            neighbor_first_edge_point_transformed = neighbor_first_edge_point_transformed.transform(&Euclid::Flip((-neighbor_point_in_self_ref).arg()));
        }

        let rotation = neighbor_first_edge_point_transformed.arg();

        Some(VertexStar::new(
            &self.point + &neighbor_point_in_self_ref,
            proto_neighbor.proto_vertex_star_index,
            parity,
            rotation,
        ))
    }

    pub fn get_proto_vertex_star<'a>(&self, tiling: &'a Tiling) -> Option<&'a ProtoVertexStar> {
        tiling.proto_vertex_stars.get(self.proto_vertex_star_index)
    }

    pub fn mutual_parity(&self, parity: bool) -> bool {
        self.parity ^ parity
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
    pub fn new(tiling: Tiling) -> Patch {
        let patch = Patch {
            tiling,
            tile_diffs: HashMap::default(),
            tiles: HashMap::default(),
            vertex_stars: HashMap::default(),
        };
        patch
    }

    pub fn drain_tile_diffs(&mut self) -> HashMap<Tile, TileDiff> {
        self.tile_diffs.drain().collect()
    }

    pub fn drain_tiles(&mut self) -> HashMap<Tile, TileDiff> {
        self.tiles.drain().into_iter().map(|(_, tile)| (tile, TileDiff::Removed)).collect()
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
            "VertexStar:\n- point: {}\n- proto_vertex_star_index: {}\n- flip: {}\n- rotation: {}",
            self.point,
            self.proto_vertex_star_index,
            self.parity,
            fmt_float(self.rotation, 3),
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
    use crate::tile::regular_polygon;
    use crate::tiling::config::{Component, Config, Neighbor, Vertex};
    use crate::tilings::_4_4_4_4;
    use geometry::Point;
    use std::f64::consts::{PI, TAU};

    #[test]
    // by link refers to asserting that all of a vertex star's neighbors (i.e. the vertex star's link) are correctly configured
    fn test_vertex_star_get_neighbor_by_link() {
        let triangle = regular_polygon(1., 3);
        let square = regular_polygon(1., 4);
        let hexagon = regular_polygon(1., 6);
        let dodecagon = regular_polygon(1., 12);
        let x = Point(1., 0.);

        let tiling_4_4_4_4 = Tiling::new(
            String::from("4.4.4.4"),
            Config(vec![Vertex {
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
        );

        let tiling_3_3_3_3_3_3 = Tiling::new(
            String::from("3.3.3.3.3.3"),
            Config(vec![Vertex {
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
        );

        let tiling_6_6_6 = Tiling::new(
            String::from("6.6.6"),
            Config(vec![Vertex {
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
        );

        let tiling_3_12_12 = Tiling::new(
            String::from("3.12.12"),
            Config(vec![Vertex {
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
        );

        let tiling_4_6_12 = Tiling::new(
            String::from("4.6.12"),
            Config(vec![Vertex {
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
        );

        for rotation in (0..8).map(|i| rad((i as f64) * TAU / 8.)) {
            println!("rotation: {}π", fmt_float(rotation / PI, 2));

            let rotate = Euclid::Rotate(rotation);
            let vertex_star = VertexStar::new(Point(0.,0.), 0, false, rotation);

            let assert_vertex_star_neighbor = |
                tiling: &Tiling,
                vertex_star: &VertexStar,
                neighbor_index: usize,
                expected_point: Point,
                expected_parity: bool,
                expected_rotation: f64,
            | {
                println!("input: {} {} | expected: {} {} {}π", vertex_star.point, neighbor_index, expected_point.transform(&rotate), expected_parity, fmt_float(rad(expected_rotation + rotation) / PI, 2));
                let neighbor_vertex_star = vertex_star.get_neighbor(tiling, neighbor_index).unwrap();
                assert_eq!(expected_point.transform(&rotate), neighbor_vertex_star.point);
                assert_eq!(expected_parity, neighbor_vertex_star.parity);
                approx_eq!(f64, rad(expected_rotation + rotation), neighbor_vertex_star.rotation);
            };

            println!("{}", tiling_4_4_4_4.name);
            assert_vertex_star_neighbor(&tiling_4_4_4_4, &vertex_star, 0, x.transform(&Euclid::Rotate(to_rad(0. * 90.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_4_4_4_4, &vertex_star, 1, x.transform(&Euclid::Rotate(to_rad(1. * 90.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_4_4_4_4, &vertex_star, 2, x.transform(&Euclid::Rotate(to_rad(2. * 90.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_4_4_4_4, &vertex_star, 3, x.transform(&Euclid::Rotate(to_rad(3. * 90.))), false, 0.);
            println!();

            println!("{}", tiling_3_3_3_3_3_3.name);
            assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &vertex_star, 0, x.transform(&Euclid::Rotate(to_rad(0. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &vertex_star, 1, x.transform(&Euclid::Rotate(to_rad(1. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &vertex_star, 2, x.transform(&Euclid::Rotate(to_rad(2. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &vertex_star, 3, x.transform(&Euclid::Rotate(to_rad(3. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &vertex_star, 4, x.transform(&Euclid::Rotate(to_rad(4. * 60.))), false, 0.);
            assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &vertex_star, 5, x.transform(&Euclid::Rotate(to_rad(5. * 60.))), false, 0.);
            println!();

            println!("{}", tiling_6_6_6.name);
            assert_vertex_star_neighbor(&tiling_6_6_6, &vertex_star, 0, x.transform(&Euclid::Rotate(to_rad(0. * 120.))), false, to_rad(60.));
            assert_vertex_star_neighbor(&tiling_6_6_6, &vertex_star, 1, x.transform(&Euclid::Rotate(to_rad(1. * 120.))), false, to_rad(60.));
            assert_vertex_star_neighbor(&tiling_6_6_6, &vertex_star, 2, x.transform(&Euclid::Rotate(to_rad(2. * 120.))), false, to_rad(60.));
            println!();

            println!("{}", tiling_3_12_12.name);
            assert_vertex_star_neighbor(&tiling_3_12_12, &vertex_star, 0, x, false, to_rad(120.));
            assert_vertex_star_neighbor(&tiling_3_12_12, &vertex_star, 1, x.transform(&Euclid::Rotate(to_rad(60.))), false, to_rad(240.));
            assert_vertex_star_neighbor(&tiling_3_12_12, &vertex_star, 2, x.transform(&Euclid::Rotate(to_rad(210.))), false, to_rad(180.));
            println!();

            println!("{}", tiling_4_6_12.name);
            assert_vertex_star_neighbor(&tiling_4_6_12, &vertex_star, 0, x, true, to_rad(180.));
            assert_vertex_star_neighbor(&tiling_4_6_12, &vertex_star, 1, x.transform(&Euclid::Rotate(to_rad(150.))), true, to_rad(120.));
            assert_vertex_star_neighbor(&tiling_4_6_12, &vertex_star, 2, x.transform(&Euclid::Rotate(to_rad(270.))), true, to_rad(0.));
            println!();
        }
    }

    #[test]
    // by chain refers to asserting that a sequence of vertex stars, the next accumulated as a neighbor of the previous star, are correctly configured
    fn test_vertex_star_get_neighbor_by_sequence() {
        let triangle = regular_polygon(1., 3);
        let square = regular_polygon(1., 4);
        let hexagon = regular_polygon(1., 6);
        let dodecagon = regular_polygon(1., 12);
        let x = Point(1., 0.);

        let tiling_4_4_4_4 = Tiling::new(
            String::from("4.4.4.4"),
            Config(vec![Vertex {
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
        );

        let tiling_3_3_3_3_3_3 = Tiling::new(
            String::from("3.3.3.3.3.3"),
            Config(vec![Vertex {
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
        );

        let tiling_6_6_6 = Tiling::new(
            String::from("6.6.6"),
            Config(vec![Vertex {
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
        );

        let tiling_3_12_12 = Tiling::new(
            String::from("3.12.12"),
            Config(vec![Vertex {
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
        );

        let tiling_4_6_12 = Tiling::new(
            String::from("4.6.12"),
            Config(vec![Vertex {
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
        );

        for rotation in (0..8).map(|i| rad((i as f64) * TAU / 8.)) {
            println!("rotation: {}π", fmt_float(rotation / PI, 2));

            let rotate = Euclid::Rotate(rotation);

            let vertex_star = VertexStar::new(Point(0.,0.), 0, false, rotation);

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

                let neighbor_vertex_star = vertex_star.get_neighbor(tiling, neighbor_index).unwrap();

                assert_eq!(expected_parity, neighbor_vertex_star.parity);
                assert_eq!(expected_point, neighbor_vertex_star.point);
                approx_eq!(f64, expected_rotation, neighbor_vertex_star.rotation);

                neighbor_vertex_star
            };

            println!("{}", tiling_4_4_4_4.name);
            let nvs = assert_vertex_star_neighbor(&tiling_4_4_4_4, &vertex_star, 0, x.transform(&Euclid::Rotate(to_rad(0. * 90.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&tiling_4_4_4_4, &nvs, 1, x.transform(&Euclid::Rotate(to_rad(1. * 90.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&tiling_4_4_4_4, &nvs, 2, x.transform(&Euclid::Rotate(to_rad(2. * 90.))), false, 0.);
            let _nvs = assert_vertex_star_neighbor(&tiling_4_4_4_4, &nvs, 3, x.transform(&Euclid::Rotate(to_rad(3. * 90.))), false, 0.);
            println!();

            println!("{}", tiling_3_3_3_3_3_3.name);
            let nvs = assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &vertex_star, 0, x.transform(&Euclid::Rotate(to_rad(0. * 60.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &nvs, 1, x.transform(&Euclid::Rotate(to_rad(1. * 60.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &nvs, 2, x.transform(&Euclid::Rotate(to_rad(2. * 60.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &nvs, 3, x.transform(&Euclid::Rotate(to_rad(3. * 60.))), false, 0.);
            let nvs = assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &nvs, 4, x.transform(&Euclid::Rotate(to_rad(4. * 60.))), false, 0.);
            let _nvs = assert_vertex_star_neighbor(&tiling_3_3_3_3_3_3, &nvs, 5, x.transform(&Euclid::Rotate(to_rad(5. * 60.))), false, 0.);
            println!();

            println!("{}", tiling_6_6_6.name);
            let nvs = assert_vertex_star_neighbor(&tiling_6_6_6, &vertex_star, 0, x.transform(&Euclid::Rotate(to_rad(0. * 120.))), false, to_rad(60.));
            let nvs = assert_vertex_star_neighbor(&tiling_6_6_6, &nvs, 1, x.transform(&Euclid::Rotate(to_rad(1. * 120.))), false, to_rad(60.));
            let _nvs = assert_vertex_star_neighbor(&tiling_6_6_6, &nvs, 2, x.transform(&Euclid::Rotate(to_rad(2. * 120.))), false, to_rad(60.));
            println!();

            println!("{}", tiling_3_12_12.name);
            let nvs = assert_vertex_star_neighbor(&tiling_3_12_12, &vertex_star, 0, x, false, to_rad(120.));
            let nvs = assert_vertex_star_neighbor(&tiling_3_12_12, &nvs, 1, x.transform(&Euclid::Rotate(to_rad(60.))), false, to_rad(240.));
            let _nvs = assert_vertex_star_neighbor(&tiling_3_12_12, &nvs, 2, x.transform(&Euclid::Rotate(to_rad(210.))), false, to_rad(180.));
            println!();

            println!("{}", tiling_4_6_12.name);

            let nvs = vertex_star.get_neighbor(&tiling_4_6_12, 0).unwrap();
            assert_eq!(true, nvs.parity);
            assert_eq!(x.transform(&rotate), nvs.point);
            approx_eq!(f64, rad(to_rad(180.) + rotation), nvs.rotation);

            let nvs = nvs.get_neighbor(&tiling_4_6_12, 1).unwrap();
            assert_eq!(false, nvs.parity);
            assert_eq!((&x + &x.transform(&Euclid::Rotate(to_rad(30.)))).transform(&rotate), nvs.point);
            approx_eq!(f64, rad(to_rad(60.) + rotation), nvs.rotation);

            let nvs = nvs.get_neighbor(&tiling_4_6_12, 2).unwrap();
            assert_eq!(true, nvs.parity);
            assert_eq!((
                &(&x + &x.transform(&Euclid::Rotate(to_rad(30.)))) + &x.transform(&Euclid::Rotate(to_rad(-30.)))
            ).transform(&rotate), nvs.point);
            approx_eq!(f64, rad(to_rad(60.) + rotation), nvs.rotation);

            println!();
        }
    }

    #[test]
    fn test_vertex_star_get_proto_vertex_star() {
        let tiling = _4_4_4_4();

        let proto_vertex_star_index = 0;
        let vertex_star = VertexStar::new(Point(0.,0.), proto_vertex_star_index, false, 0.);
        let proto_vertex_star = match vertex_star.get_proto_vertex_star(&tiling) { None => return assert!(false), Some(pvs) => pvs };
        assert_eq!(proto_vertex_star.index, proto_vertex_star_index);

        let proto_vertex_star_index = 1;
        let vertex_star = VertexStar::new(Point(0.,0.), proto_vertex_star_index, false, 0.);
        match vertex_star.get_proto_vertex_star(&tiling) { None => assert!(true), Some(_) => assert!(false) };
    }

    #[test]
    fn test_vertex_star_mutual_parity() {
        let vertex_star = VertexStar::new(Point(0.,0.), 0, false, 0.);
        assert_eq!(false, vertex_star.mutual_parity(false));
        assert_eq!(true, vertex_star.mutual_parity(true));

        let vertex_star = VertexStar::new(Point(0.,0.), 0, true, 0.);
        assert_eq!(true, vertex_star.mutual_parity(false));
        assert_eq!(false, vertex_star.mutual_parity(true));
    }
}
