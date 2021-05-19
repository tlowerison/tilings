use common::{approx_eq, hash_float, rad, rev_iter};
use geometry::{reduce_transforms, Euclid, Generator, Point, Transform, Transformable};
use itertools::izip;
use std::{
    collections::VecDeque,
    f64::consts::{PI, TAU},
    hash::{Hash, Hasher},
    iter,
};

#[derive(Clone)]
pub struct ProtoTile {
    pub points: VecDeque<Point>,
    pub parity: bool,
}

impl ProtoTile {
    pub fn new(tuples: Vec<(f64, f64)>) -> ProtoTile {
        assert!(tuples.len() > 2);
        let mut points = tuples
            .into_iter()
            .map(|(x, y)| Point(x, y))
            .collect::<VecDeque<Point>>();
        points.shrink_to_fit();
        ProtoTile {
            points,
            parity: false,
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
        let point = match self.points.get(point_index) {
            Some(point) => point,
            None => panic!(
                "failed to find angle: index is out of bounds for ProtoTile {}",
                self
            ),
        };
        let point1 = match self.points.get((point_index + (size - 1)) % size) {
            Some(point) => point,
            None => panic!(
                "failed to find angle: preceding index is out of bounds for ProtoTile {}",
                self
            ),
        };
        let point2 = match self.points.get((point_index + 1) % size) {
            Some(point) => point,
            None => panic!(
                "failed to find angle: succeeding index is out of bounds for ProtoTile {}",
                self
            ),
        };
        let angle = Point::angle(point1, point, point2);
        rad(if self.parity { TAU - angle } else { angle })
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

        let actual_side_lengths: Vec<f64> = self
            .points
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let b = match self.points.get((i + 1) % size) {
                    Some(point) => point,
                    None => panic!("could not find point {} for ProtoTile {}", i, self),
                };
                (b - a).norm()
            })
            .collect();

        let max_exp = match exp_side_lengths
            .iter()
            .reduce(|max, side| if side > max { side } else { max })
        {
            Some(max) => max,
            None => panic!(
                "couldn't calc max actual side length for Prototile {}",
                self
            ),
        };
        let max_actual =
            match actual_side_lengths
                .iter()
                .reduce(|max, side| if side > max { side } else { max })
            {
                Some(max) => max,
                None => panic!(
                    "couldn't calc max expected side length for Prototile {}",
                    self
                ),
            };

        for (exp, actual) in izip!(exp_side_lengths.iter(), actual_side_lengths.iter()) {
            approx_eq!(f64, exp / max_exp, actual / max_actual);
        }
    }

    pub fn centroid(&self) -> Point {
        // calc area
        let mut points = self.points.clone();
        points.rotate_right(1);
        let terms = izip!(self.points.iter(), points.iter())
            .map(|(p0, p1)| {
                let conv = p0.0 * p1.1 - p0.1 * p1.0;
                (conv, conv * (p0.0 + p1.0), conv * (p0.1 + p1.1))
            })
            .reduce(|(a0, a1, a2), (e0, e1, e2)| (a0 + e0, a1 + e1, a2 + e2))
            .unwrap();
        let area = terms.0 / 2.;
        Point(terms.1 / (6. * area), terms.2 / (6. * area))
    }

    pub fn size(&self) -> usize {
        self.points.len()
    }
}

impl Eq for ProtoTile {}

// hash angles up to two decimals
impl Hash for ProtoTile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for i in rev_iter(self.parity, 0..self.size()) {
            hash_float(self.angle(i), 2).hash(state);
        }
    }
}

impl PartialEq for ProtoTile {
    fn eq(&self, other: &Self) -> bool {
        if self.size() != other.size() {
            return false;
        }
        for (self_i, other_i) in izip!(
            rev_iter(self.parity, 0..self.size()),
            rev_iter(other.parity, 0..other.size())
        ) {
            if hash_float(self.angle(self_i), 4) != hash_float(other.angle(other_i), 4) {
                return false;
            }
        }
        return true;
    }
}

impl<'a> Transformable<'a> for ProtoTile {
    fn transform<T: Transform>(&self, transform: &'a T) -> ProtoTile {
        let affine = transform.as_affine();
        ProtoTile {
            points: self
                .points
                .iter()
                .map(|point| point.transform(&affine))
                .collect(),
            parity: self.parity ^ affine.is_flip(),
        }
    }
}

impl std::fmt::Display for ProtoTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.points
                .iter()
                .map(|point| format!("{}", point))
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

#[derive(Clone)]
pub struct Tile {
    pub points: Vec<Point>,
    pub centroid: Point,
    pub parity: bool,
}

impl Tile {
    pub fn new(proto_tile: ProtoTile, parity: bool) -> Tile {
        let centroid = proto_tile.centroid();
        Tile {
            centroid,
            parity,
            points: proto_tile.points.into_iter().collect::<Vec<Point>>(),
        }
    }

    pub fn closest_edge(&self, point: &Point) -> (Point, Point) {
        let edges = izip!(
            self.points.iter(),
            self.points.iter().skip(1).chain(self.points.first())
        );
        let mut min = f64::MAX;
        let mut closest_edge = (self.points.get(0).unwrap(), self.points.get(1).unwrap());
        for (start, stop) in edges {
            // Consider the line extending the edge, parameterized as start + t * (stop - start).
            // We find projection of point onto the line.
            // It falls where t = [(point-start) . (stop-start)] / |stop-start|^2
            // Clamp t in [0,1] to handle points outside the edge.
            let edge = stop - start;
            let t = ((point - start).dot(&edge) / edge.norm_squared()).clamp(0., 1.);
            let projection = start + &edge.mul(t);
            let distance = (point - &projection).norm();
            if distance < min {
                min = distance;
                closest_edge = (start, stop);
            }
        }
        if !self.parity {
            (closest_edge.0.clone(), closest_edge.1.clone())
        } else {
            (closest_edge.1.clone(), closest_edge.0.clone())
        }
    }

    pub fn size(&self) -> usize {
        self.points.len()
    }
}

impl Eq for Tile {}

impl Hash for Tile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        hash_float(self.centroid.0, 2).hash(state);
        hash_float(self.centroid.1, 2).hash(state);
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.centroid == other.centroid
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.points
                .iter()
                .map(|point| format!("{}", point))
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

pub fn regular_polygon(side_length: f64, num_sides: usize) -> ProtoTile {
    let n = num_sides as f64;
    let centroid_angle_of_inclination = PI * (0.5 - 1. / n);
    let radius = side_length / 2. / centroid_angle_of_inclination.cos();
    let centroid = Point(
        radius * centroid_angle_of_inclination.cos(),
        radius * centroid_angle_of_inclination.sin(),
    );

    let affine = reduce_transforms(vec![
        &Euclid::Translate((-centroid).values()),
        &Euclid::Rotate(TAU / n),
        &Euclid::Translate(centroid.values()),
    ]);

    let mut generator = Generator::new(affine);

    let proto_tile = ProtoTile {
        points: iter::repeat(Point(0., 0.))
            .take(num_sides)
            .enumerate()
            .map(|(i, point)| point.transform(&generator(i)))
            .collect(),
        parity: false,
    };

    proto_tile.assert_angles(
        iter::repeat(2. * centroid_angle_of_inclination)
            .take(num_sides)
            .collect(),
    );
    proto_tile.assert_sides(iter::repeat(side_length).take(num_sides).collect());

    proto_tile
}

#[cfg(test)]
mod tests {
    use super::*;
    use geometry::Point;

    #[test]
    fn test_tile_closest_edge() {
        let square = Tile::new(ProtoTile::new(vec![(0., 0.), (1., 0.), (1., 1.), (0., 1.)]), false);

        let edge = square.closest_edge(&Point::new((0.5, -0.5)));
        assert_eq!(Point(0., 0.), edge.0);
        assert_eq!(Point(1., 0.), edge.1);

        let edge = square.closest_edge(&Point::new((1.5, 0.5)));
        assert_eq!(Point(1., 0.), edge.0);
        assert_eq!(Point(1., 1.), edge.1);

        let edge = square.closest_edge(&Point::new((0.5, 1.5)));
        assert_eq!(Point(1., 1.), edge.0);
        assert_eq!(Point(0., 1.), edge.1);

        let edge = square.closest_edge(&Point::new((-0.5, 0.5)));
        assert_eq!(Point(0., 1.), edge.0);
        assert_eq!(Point(0., 0.), edge.1);
    }

    // #[test]
    // fn test_regular_polygon() {
    //     let triangle = regular_polygon(1., 3);
    //     let square = regular_polygon(1., 4);
    //     let hexagon = regular_polygon(1., 6);
    //     println!("{}", triangle);
    //     println!("{}", square);
    //     println!("{}", hexagon);
    //     assert!(false);
    // }
}
