use common::{approx_eq, DEFAULT_PRECISION, fmt_float, hash_float, rad, rev_iter};
use geometry::{reduce_transforms, Euclid, Generator, Point, Transform, Transformable};
use itertools::{interleave, Itertools, izip};
use models::FullPolygon;
use std::{
    f64::consts::{PI, TAU},
    hash::{Hash, Hasher},
    iter,
};

#[derive(Clone, Debug)]
pub struct ProtoTile {
    pub points: Vec<Point>,
    pub parity: bool,
}

impl ProtoTile {
    pub fn new(mut points: Vec<Point>) -> ProtoTile {
        assert!(points.len() > 2);
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
        let points = &self.points;
        let size = points.len();
        assert!(point_index < size);

        let point = match points.get(point_index) {
            Some(point) => point,
            None => panic!(
                "failed to find angle: index is out of bounds for ProtoTile {}",
                self
            ),
        };
        let point1 = match points.get((point_index + (size - 1)) % size) {
            Some(point) => point,
            None => panic!(
                "failed to find angle: preceding index is out of bounds for ProtoTile {}",
                self
            ),
        };
        let point2 = match points.get((point_index + 1) % size) {
            Some(point) => point,
            None => panic!(
                "failed to find angle: succeeding index is out of bounds for ProtoTile {}",
                self
            ),
        };
        let angle = Point::angle(point1, point, point2);
        rad(if self.parity { TAU - angle } else { angle })
    }

    pub fn angles(&self) -> Vec<f64> {
        (0..self.size()).map(|point_index| self.angle(point_index)).collect()
    }

    pub fn angles_str(&self) -> String {
        self.angles().into_iter().map(|angle| format!("{}", fmt_float(angle / TAU * 360., 2))).collect::<Vec<String>>().join(" ")
    }

    // assert_angles asserts that all angles equal those provided
    pub fn assert_angles(&self, angles: Vec<f64>) {
        let exp_angles = self.angles();
        assert_eq!(exp_angles.len(), angles.len());
        for (exp_angle, angle) in izip!(exp_angles, angles) {
            approx_eq!(f64, exp_angle, angle);
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

impl From<&FullPolygon> for ProtoTile {
    fn from(full_polygon: &FullPolygon) -> ProtoTile {
        ProtoTile::new(
            full_polygon
                .points
                .iter()
                .map(|point| Point(point.point.x, point.point.y))
                .collect()
        )
    }
}

impl Eq for ProtoTile {}

// hash angles up to two decimals
impl Hash for ProtoTile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for i in rev_iter(self.parity, 0..self.size()) {
            hash_float(self.angle(i), DEFAULT_PRECISION).hash(state);
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
            if hash_float(self.angle(self_i), DEFAULT_PRECISION) != hash_float(other.angle(other_i), DEFAULT_PRECISION) {
                return false;
            }
        }
        return true;
    }
}

impl<'a> Transformable<'a> for ProtoTile {
    fn transform<T: Transform>(&self, transform: &'a T) -> Self {
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
        hash_float(self.centroid.0, DEFAULT_PRECISION).hash(state);
        hash_float(self.centroid.1, DEFAULT_PRECISION).hash(state);
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.centroid == other.centroid
    }
}

impl<'a> Transformable<'a> for Tile {
    fn transform<T: Transform>(&self, transform: &'a T) -> Self {
        let affine = transform.as_affine();
        Tile {
            points: self
                .points
                .iter()
                .map(|point| point.transform(&affine))
                .collect(),
            centroid: self.centroid.transform(&affine),
            parity: self.parity ^ affine.is_flip(),
        }
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
    if num_sides < 3 || side_length <= 0. {
        panic!("invalid regular polygon: side_length = {}, num_sides = {}", side_length, num_sides);
    }

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

    let proto_tile = ProtoTile::new(
        iter::repeat(Point(0., 0.))
            .take(num_sides)
            .enumerate()
            .map(|(i, point)| point.transform(&generator(i)))
            .collect(),
    );

    proto_tile.assert_angles(
        iter::repeat(2. * centroid_angle_of_inclination)
            .take(num_sides)
            .collect(),
    );
    proto_tile.assert_sides(iter::repeat(side_length).take(num_sides).collect());

    proto_tile
}

// star_polygon returns a ProtoTile with 2 * num_base_sides points, where each point which
// was included in the original regular polygon has its internal angle set to the provided value.
pub fn star_polygon(side_length: f64, num_base_sides: usize, internal_angle: f64) -> ProtoTile {
    if num_base_sides < 3 {
        panic!("invalid star polygon: side_length = {}, num_sides = {}", side_length, num_base_sides);
    }
    if internal_angle <= 0. || internal_angle >= PI {
        panic!("invalid star polygon: expected internal angle to be be in the open interval (0, π) but received {}π", internal_angle / PI);
    }
    let internal_angle_diff = (PI - TAU / (num_base_sides as f64) - internal_angle) / 2.;
    let base = regular_polygon(2. * side_length * internal_angle_diff.cos(), num_base_sides);
    let x = Point(side_length, 0.);
    let indented_points = base.points
        .iter()
        .enumerate()
        .map(|(i, point)| {
            let next_point = base.points.get((i + 1) % base.size()).unwrap();
            let rotation = (next_point - &point).arg();
            point + &x.transform(&Euclid::Rotate(rotation + internal_angle_diff))
        })
        .collect_vec();

    let mut dented_points = interleave(base.points.clone().into_iter(), indented_points.into_iter()).collect_vec();
    dented_points.shrink_to_fit();

    ProtoTile::new(dented_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{fmt_float, to_rad};
    use geometry::{ORIGIN, Point};

    const X: Point = Point(1., 0.);

    #[test]
    fn test_tile_closest_edge() {
        let square = Tile::new(regular_polygon(1., 4), false);

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

    #[test]
    fn test_regular_polygon() {
        for num_sides in 3..100 {
            println!("num sides: {}", num_sides);
            let polygon = regular_polygon(1., num_sides);
            let mut point = ORIGIN.clone();
            let exterior_angle = to_rad(360. / (num_sides as f64));
            println!("exterior angle: {}π", fmt_float(exterior_angle / PI, 2));
            for i in 0..num_sides {
                println!("point {}", i);
                approx_eq!(&Point, &point, polygon.points.get(i).unwrap());
                point = &point + &X.transform(&Euclid::Rotate((i as f64) * exterior_angle));
            }
        }
    }

    #[test]
    fn test_star_polygon() {
        let three_two_star = star_polygon(1., 3, to_rad(30.));
        assert_eq!(6, three_two_star.size());

        let internal_angle_diff = to_rad(15.);
        let exterior_angle = to_rad(120.);
        let mut point = ORIGIN;

        approx_eq!(&Point, &point, three_two_star.points.get(0).unwrap()); point = &point + &X.transform(&Euclid::Rotate(0. * exterior_angle + 1. * internal_angle_diff));
        approx_eq!(&Point, &point, three_two_star.points.get(1).unwrap()); point = &point + &X.transform(&Euclid::Rotate(0. * exterior_angle - 1. * internal_angle_diff));
        approx_eq!(&Point, &point, three_two_star.points.get(2).unwrap()); point = &point + &X.transform(&Euclid::Rotate(1. * exterior_angle + 1. * internal_angle_diff));
        approx_eq!(&Point, &point, three_two_star.points.get(3).unwrap()); point = &point + &X.transform(&Euclid::Rotate(1. * exterior_angle - 1. * internal_angle_diff));
        approx_eq!(&Point, &point, three_two_star.points.get(4).unwrap()); point = &point + &X.transform(&Euclid::Rotate(2. * exterior_angle + 1. * internal_angle_diff));
        approx_eq!(&Point, &point, three_two_star.points.get(5).unwrap());

        let six_two_star = star_polygon(1., 6, to_rad(60.));
        assert_eq!(12, six_two_star.size());

        let internal_angle_diff = to_rad(30.);
        let exterior_angle = to_rad(60.);
        let mut point = ORIGIN;

        approx_eq!(&Point, &point, six_two_star.points.get(0).unwrap()); point = &point + &X.transform(&Euclid::Rotate(0. * exterior_angle + 1. * internal_angle_diff));
        approx_eq!(&Point, &point, six_two_star.points.get(1).unwrap()); point = &point + &X.transform(&Euclid::Rotate(0. * exterior_angle - 1. * internal_angle_diff));
        approx_eq!(&Point, &point, six_two_star.points.get(2).unwrap()); point = &point + &X.transform(&Euclid::Rotate(1. * exterior_angle + 1. * internal_angle_diff));
        approx_eq!(&Point, &point, six_two_star.points.get(3).unwrap()); point = &point + &X.transform(&Euclid::Rotate(1. * exterior_angle - 1. * internal_angle_diff));
        approx_eq!(&Point, &point, six_two_star.points.get(4).unwrap()); point = &point + &X.transform(&Euclid::Rotate(2. * exterior_angle + 1. * internal_angle_diff));
        approx_eq!(&Point, &point, six_two_star.points.get(5).unwrap()); point = &point + &X.transform(&Euclid::Rotate(2. * exterior_angle - 1. * internal_angle_diff));
        approx_eq!(&Point, &point, six_two_star.points.get(6).unwrap()); point = &point + &X.transform(&Euclid::Rotate(3. * exterior_angle + 1. * internal_angle_diff));
        approx_eq!(&Point, &point, six_two_star.points.get(7).unwrap()); point = &point + &X.transform(&Euclid::Rotate(3. * exterior_angle - 1. * internal_angle_diff));
        approx_eq!(&Point, &point, six_two_star.points.get(8).unwrap()); point = &point + &X.transform(&Euclid::Rotate(4. * exterior_angle + 1. * internal_angle_diff));
        approx_eq!(&Point, &point, six_two_star.points.get(9).unwrap()); point = &point + &X.transform(&Euclid::Rotate(4. * exterior_angle - 1. * internal_angle_diff));
        approx_eq!(&Point, &point, six_two_star.points.get(10).unwrap()); point = &point + &X.transform(&Euclid::Rotate(5. * exterior_angle + 1. * internal_angle_diff));
        approx_eq!(&Point, &point, six_two_star.points.get(11).unwrap());
    }
}
