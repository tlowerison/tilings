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
        let proto_vertex_star = match self.get_proto_vertex_star(tiling) { None => return None, Some(pvs) => pvs };
        let proto_neighbor_index = if !self.parity { index } else { (proto_vertex_star.size() - index) % proto_vertex_star.size() };
        let proto_neighbor = proto_vertex_star.proto_neighbors.get(proto_neighbor_index).unwrap();
        let neighbor_proto_vertex_star = match tiling.proto_vertex_stars.get(proto_neighbor.proto_vertex_star_index) { None => return None, Some(pvs) => pvs };

        let neighbor_point_in_self_ref = Point::new(proto_neighbor.transform.translate);
        let self_point_in_neighbor_ref = match neighbor_proto_vertex_star.proto_neighbors.get(proto_neighbor.neighbor_index) {
            None => return None,
            Some(pn) => Point::new(pn.transform.translate),
        };

        let angle_1 = (-neighbor_point_in_self_ref).arg();
        let angle_2 = self_point_in_neighbor_ref.arg();

        let parity = self.mutual_parity(proto_neighbor.transform.parity);
        let rotation = rad(self.rotation + angle_1 - (if !parity { angle_2 } else { -angle_2 }));

        Some(VertexStar::new(
            &self.point + &neighbor_point_in_self_ref.transform(&Euclid::Rotate(self.rotation)),
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
    use common::approx_eq;
    use geometry::Point;
    use std::f64::consts::{PI, TAU};

    #[test]
    fn test_vertex_star_get_neighbor() {
        let s32 = 3_f64.sqrt() / 2.;
        let triangle = regular_polygon(1., 3);
        let square = regular_polygon(1., 4);
        let hexagon = regular_polygon(1., 6);
        let dodecagon = regular_polygon(1., 12);

        for rotation in (0..8).map(|i| rad((i as f64) * TAU / 8.)) {
            println!("rotation: {}π", fmt_float(rotation / PI, 2));

            let rotate = Euclid::Rotate(rotation);

            let tiling = Tiling::new(
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
            let vertex_star = VertexStar::new(Point(0.,0.), 0, false, rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 0) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(1., 0.).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(0. + rotation), neighbor_vertex_star.rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 1) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(0., 1.).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(0. + rotation), neighbor_vertex_star.rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 2) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(-1., 0.).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(0. + rotation), neighbor_vertex_star.rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 3) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(0., -1.).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(0. + rotation), neighbor_vertex_star.rotation);


            let tiling = Tiling::new(
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
            let vertex_star = VertexStar::new(Point(0.,0.), 0, false, rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 0) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(1., 0.).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(0. + rotation), neighbor_vertex_star.rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 1) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(0.5, s32).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(0. + rotation), neighbor_vertex_star.rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 2) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(-0.5, s32).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(0. + rotation), neighbor_vertex_star.rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 3) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(-1., 0.).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(0. + rotation), neighbor_vertex_star.rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 4) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(-0.5, -s32).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(0. + rotation), neighbor_vertex_star.rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 5) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(0.5, -s32).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(0. + rotation), neighbor_vertex_star.rotation);


            let tiling = Tiling::new(
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
            let vertex_star = VertexStar::new(Point(0.,0.), 0, false, rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 0) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(1., 0.).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(1. * PI / 3. + rotation), neighbor_vertex_star.rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 1) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(-0.5, s32).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(1. * PI / 3. + rotation), neighbor_vertex_star.rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 2) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(-0.5, -s32).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(1. * PI / 3. + rotation), neighbor_vertex_star.rotation);


            let tiling = Tiling::new(
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
            let vertex_star = VertexStar::new(Point(0.,0.), 0, false, rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 0) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(1., 0.).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(2. * PI / 3. + rotation), neighbor_vertex_star.rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 1) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(0.5, s32).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(240. / 360. * TAU + rotation), neighbor_vertex_star.rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 2) { None => return assert!(false), Some(vs) => vs };
            let angle = 210. / 360. * TAU;
            assert_eq!(Point(angle.cos(), angle.sin()).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(false, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(PI + rotation), neighbor_vertex_star.rotation);


            let tiling = Tiling::new(
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
            let vertex_star = VertexStar::new(Point(0.,0.), 0, false, rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 0) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(1., 0.).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(true, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(PI + rotation), neighbor_vertex_star.rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 1) { None => return assert!(false), Some(vs) => vs };
            let angle = 150. / 360. * TAU;
            assert_eq!(Point(angle.cos(), angle.sin()).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(true, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(2. * PI / 3. + rotation), neighbor_vertex_star.rotation);

            let neighbor_vertex_star = match vertex_star.get_neighbor(&tiling, 2) { None => return assert!(false), Some(vs) => vs };
            assert_eq!(Point(0., -1.).transform(&rotate), neighbor_vertex_star.point);
            assert_eq!(true, neighbor_vertex_star.parity);
            approx_eq!(f64, rad(0. + rotation), neighbor_vertex_star.rotation);
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
