use float_cmp::*;
use itertools::*;
use std::{
    collections::VecDeque,
    f64::consts::TAU,
    hash::{Hash, Hasher},
};
use crate::common::*;

pub struct ProtoTile {
    pub points: VecDeque<Point>,
    pub flipped: bool,
}

impl ProtoTile {
    pub fn new(tuples: Vec<(f64, f64)>) -> ProtoTile {
        assert!(tuples.len() > 2);
        let mut points = tuples.into_iter().map(|(x,y)| Point(x,y)).collect::<VecDeque<Point>>();
        points.shrink_to_fit();
        ProtoTile {
            points,
            flipped: false,
        }
    }

    // reorient_about_origin shifts the underlying points of a ProtoTile so that the first
    // point is the closest to the origin
    pub fn reorient(&mut self, origin: &Point) {
        let mut argmin = 0_usize;
        let mut min = f64::MAX;
        for (i, point) in self.points.iter().enumerate() {
            let norm = (point - origin).norm();
            if norm < min {
                argmin = i;
                min = norm;
            }
        }
        self.points.rotate_left(argmin);
    }

    // angle returns the angle in radians between the line segments drawn between (point_index-1,point_index) and (point_index,point_index+1)
    pub fn angle(&self, point_index: usize) -> f64 {
        let size = self.size();
        assert!(point_index < size);
        let point = match self.points.get(point_index) { Some(point) => point, None => panic!("failed to find angle: index is out of bounds for ProtoTile {}", self) };
        let point1 = match self.points.get((point_index + (size - 1)) % size) { Some(point) => point, None => panic!("failed to find angle: preceding index is out of bounds for ProtoTile {}", self) };
        let point2 = match self.points.get((point_index + 1) % size) { Some(point) => point, None => panic!("failed to find angle: succeeding index is out of bounds for ProtoTile {}", self) };
        let angle = (point1 - point).arg() - (point2 - point).arg();
        (if self.flipped { TAU - angle } else { angle } + TAU) % TAU
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

        let actual_side_lengths: Vec<f64> = self.points.iter().enumerate().map(|(i, a)| {
            let b = match self.points.get((i + 1) % size) { Some(point) => point, None => panic!("could not find point {} for ProtoTile {}", i, self) };
            (b - a).norm()
        }).collect();

        let max_exp = match exp_side_lengths.iter().reduce(|max,side| if side > max { side } else { max }) { Some(max) => max, None => panic!("couldn't calc max actual side length for Prototile {}", self) };
        let max_actual = match actual_side_lengths.iter().reduce(|max,side| if side > max { side } else { max }) { Some(max) => max, None => panic!("couldn't calc max expected side length for Prototile {}", self) };

        for (exp, actual) in izip!(exp_side_lengths.iter(), actual_side_lengths.iter()) {
            approx_eq!(f64, exp / max_exp, actual / max_actual);
        }
    }

    pub fn centroid(&self) -> Point {
        // calc area
        let mut points = self.points.clone();
        points.rotate_right(1);
        let terms = izip!(self.points.iter(), points.iter())
            .map(|(p0,p1)| {
                let conv = p0.0 * p1.1 - p0.1 * p1.0;
                (conv, conv * (p0.0 + p1.0), conv * (p0.1 + p1.1))
            })
            .reduce(|(a0,a1,a2),(e0,e1,e2)| (a0+e0,a1+e1,a2+e2))
            .unwrap();
        let area = terms.0 / 2.;
        Point(terms.1 / (6. * area), terms.2 / (6. * area))
    }

    pub fn size(&self) -> usize {
        self.points.len()
    }
}

impl Eq for ProtoTile {}

impl PartialEq for ProtoTile {
    fn eq(&self, other: &Self) -> bool {
        if self.size() != other.size() {
            return false
        }
        for (self_i, other_i) in izip!(range_iter(0..self.size(), self.flipped), range_iter(0..other.size(), other.flipped)) {
            let self_angle = approx_f64(self.angle(self_i));
            let other_angle = approx_f64(other.angle(other_i));
            if self_angle != other_angle {
                return false
            }
        }
        return true
    }
}

// hash angles up to two decimals
impl Hash for ProtoTile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for i in range_iter(0..self.size(), self.flipped) {
            approx_f64(self.angle(i)).hash(state);
        }
    }
}

impl Clone for ProtoTile {
    fn clone(&self) -> Self {
        ProtoTile {
            points: self.points.clone(),
            flipped: self.flipped,
        }
    }
}

impl<'a> Transformable<'a> for ProtoTile {
    fn transform<T: Transform>(&self, transform: &'a T) -> ProtoTile {
        let affine = transform.as_affine();
        ProtoTile {
            points: self.points.iter().map(|point| point.transform(&affine)).collect(),
            flipped: self.flipped ^ affine.is_flip(),
        }
    }
}

impl std::fmt::Display for ProtoTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"[{}]", self.points.iter().map(|point| format!("{}", point)).collect::<Vec<String>>().join(","))
    }
}

#[derive(Clone)]
pub struct Tile {
    pub proto_tile: ProtoTile,
    pub centroid: Point,
}

impl Tile {
    pub fn new(proto_tile: ProtoTile) -> Tile {
        let centroid = proto_tile.centroid();
        Tile {
            proto_tile,
            centroid,
        }
    }
}

impl Eq for Tile {}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.centroid == other.centroid
    }
}

impl std::hash::Hash for Tile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        approx_f64(self.centroid.0).hash(state);
        approx_f64(self.centroid.1).hash(state);
    }
}
