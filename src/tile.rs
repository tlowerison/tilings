use crate::common::*;
use core::fmt;
use float_cmp::*;
use itertools::*;
use std::f64::consts::TAU;
use svg::node::element::path::{Command, Data, Parameters, Position};

pub struct ProtoTile(pub(crate) Vec<Point>);

impl Clone for ProtoTile {
    fn clone(&self) -> Self {
        ProtoTile(self.0.clone())
    }
}

impl<'a> Transformable<'a> for ProtoTile {
    fn transform<T: Transform>(&self, transform: &'a T) -> ProtoTile {
        ProtoTile(self.0.iter().map(|point| point.transform(&transform.as_affine())).collect())
    }
}

const NULL_PARAMETERS_INDEX: usize = 100;

impl ProtoTile {
    pub fn new(points: Vec<(f64, f64)>) -> ProtoTile {
        ProtoTile(points.into_iter().map(|(x,y)| Point(x,y)).collect())
    }

    // parses an SVG.path.data object into a ProtoTile
    // It extracts the ProtoTile's points from the provided path. In order
    // to enforce a continuous boundary requirement for all ProtoTiles,
    // Data.move_to is disallowed; this requirement means
    // that the starting point of the ProtoTile will always be (0,0).
    // All "points" of the ProtoTile should be marked with the use of Data.move_by((0,0)).
    // In order to support nonlinear lines, Data.close is also disallowed,
    // so all ProtoTile's must finish with some attribute whose end position is (0,0).
    pub fn new_from_svg(data: Data) -> ProtoTile {
        let mut cur_point = ORIGIN;
        let mut points: Vec<Point> = vec![ORIGIN];
        for datum in data.iter() {
            match datum {
                Command::Move(position, parameters) => {
                    match position {
                        Position::Absolute => panic!("Data.move_to is disallowed - ProtoTiles must have continuous boundary"),
                        Position::Relative => {
                            let x = get_parameter(parameters, 0);
                            let y = get_parameter(parameters, 1);
                            if x != 0. || y != 0. {
                                panic!("Data.move_by must be to (0.,0.) - ProtoTiles must have continuous boundary")
                            }
                            points.push(Point(cur_point.0, cur_point.1));
                        },
                    }
                },
                Command::Close => {panic!("Data.close is disallowed - ProtoTiles must specify the closing line to (0., 0.)")},
                Command::Line(position, parameters) => update_cur_point(position, parameters, &mut cur_point, 0, 1),
                Command::HorizontalLine(position, parameters) => update_cur_point(position, parameters, &mut cur_point, 0, NULL_PARAMETERS_INDEX),
                Command::VerticalLine(position, parameters) => update_cur_point(position, parameters, &mut cur_point, NULL_PARAMETERS_INDEX, 0),
                Command::QuadraticCurve(position, parameters) => update_cur_point(position, parameters, &mut cur_point, 2, 3),
                Command::SmoothQuadraticCurve(position, parameters) => update_cur_point(position, parameters, &mut cur_point, 0, 1),
                Command::CubicCurve(position, parameters) => update_cur_point(position, parameters, &mut cur_point, 4, 5),
                Command::SmoothCubicCurve(position, parameters) => update_cur_point(position, parameters, &mut cur_point, 2, 3),
                Command::EllipticalArc(position, parameters) => update_cur_point(position, parameters, &mut cur_point, 5, 6),
            }
        }
        ProtoTile(points)
    }

    // angle returns the angle in degrees between the line segments drawn between (point_index-1,point_index) and (point_index,point_index+1)
    pub fn angle(&self, point_index: usize) -> f64 {
        let size = self.size();
        assert!(point_index < size);
        let point = match self.0.get(point_index) { Some(point) => point, None => panic!("failed to find angle: index is out of bounds for ProtoTile {}", self) };
        let point1 = match self.0.get((point_index + (size - 1)) % size) { Some(point) => point, None => panic!("failed to find angle: preceding index is out of bounds for ProtoTile {}", self) };
        let point2 = match self.0.get((point_index + 1) % size) { Some(point) => point, None => panic!("failed to find angle: succeeding index is out of bounds for ProtoTile {}", self) };
        let p1 = point1 - point;
        let p2 = point2 - point;
        ((p1.1.atan2(p1.0) - p2.1.atan2(p2.0)) / TAU * TAUD + TAUD) % TAUD
    }

    // assert_angles asserts that all angles equal those provided
    pub fn assert_angles(&self, angles: Vec<f64>) {
        assert_eq!(self.size(), angles.len());
        for (point_index, angle) in angles.into_iter().enumerate() {
            approx_eq!(f64, angle, self.angle(point_index));
        }
    }

    // assert_sides asserts that all side lengths are proportionally correct relative to themselves
    // side[0] refers to the edge connecting point[0] and point[1]
    pub fn assert_sides(&self, exp_side_lengths: Vec<f64>) {
        let size = self.size();
        assert_eq!(size, exp_side_lengths.len());

        let actual_side_lengths: Vec<f64> = self.0.iter().enumerate().map(|(i, a)| {
            let b = match self.0.get((i + 1) % size) { Some(point) => point, None => panic!("could not find point {} for ProtoTile {}", i, self) };
            (b - a).norm()
        }).collect();

        let max_exp = match exp_side_lengths.iter().reduce(|max,side| if side > max { side } else { max }) { Some(max) => max, None => panic!("couldn't calc max actual side length for Prototile {}", self) };
        let max_actual = match actual_side_lengths.iter().reduce(|max,side| if side > max { side } else { max }) { Some(max) => max, None => panic!("couldn't calc max expected side length for Prototile {}", self) };

        for (exp, actual) in izip!(exp_side_lengths.iter(), actual_side_lengths.iter()) {
            approx_eq!(f64, exp / max_exp, actual / max_actual);
        }
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }
}

fn update_cur_point(position: &Position, parameters: &Parameters, cur_point: &mut Point, x_index: usize, y_index: usize) {
    match position {
        Position::Absolute => {
            if x_index != NULL_PARAMETERS_INDEX { cur_point.0 = get_parameter(parameters, x_index) }
            if y_index != NULL_PARAMETERS_INDEX { cur_point.1 = get_parameter(parameters, y_index) }
        }
        Position::Relative => {
            if x_index != NULL_PARAMETERS_INDEX { cur_point.0 += get_parameter(parameters, x_index) }
            if y_index != NULL_PARAMETERS_INDEX { cur_point.1 += get_parameter(parameters, y_index) }
        }
    }
}

fn get_parameter(parameters: &Parameters, index: usize) -> f64 {
    match (*parameters).get(index) { Some(parameter) => *parameter as f64, None => panic!("could not deref parameters.{}", index) }
}

impl fmt::Display for ProtoTile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"[{}]", self.0.as_slice().iter().map(|point| format!("{}", point)).collect::<Vec<String>>().join(","))
    }
}

pub struct Tile {
    pub proto_tile: ProtoTile,
    pub affines: Vec<Affine>,
}

impl Tile {
    pub fn new(proto_tile: ProtoTile) -> Tile {
        Tile {
            proto_tile,
            affines: vec![IDENTITY_AFFINE],
        }
    }
}

impl<'a> DelayedTransformable<'a> for Tile {
    type Item = ProtoTile;

    fn add<T: Transform>(&mut self, transform: &'a T) -> &Self {
        self.affines.push(transform.as_affine());
        self
    }

    fn reduce(&self) -> Self::Item {
        ProtoTile(self.proto_tile.transform(&reduce_transforms(&self.affines)).0)
    }
}

pub struct TileSet(pub(crate) Vec<ProtoTile>);
