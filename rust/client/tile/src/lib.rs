use common::{approx_eq, DEFAULT_PRECISION, hash_float, rad};
use geometry::{reduce_transforms, Bounds, Edge, Euclid, Generator, Point, Spatial, Transform, Transformable};
use itertools::{interleave, Itertools, izip};
use std::{f64::consts::{PI, TAU}, hash::Hash, iter};

#[derive(Clone, Debug)]
pub struct Tile {
    pub points: Vec<Point>,
    pub centroid: Point,
    pub parity: bool,
}

impl Tile {
    pub fn new(mut points: Vec<Point>) -> Tile {
        assert!(points.len() > 2);
        points.shrink_to_fit();
        let centroid = Tile::centroid(&points);
        Tile {
            points,
            centroid,
            parity: false,
        }
    }

    // angle returns the angle in radians between the line segments drawn between (point_index-1,point_index) and (point_index,point_index+1)
    pub fn angle(&self, point_index: usize) -> f64 {
        let points = &self.points;
        let size = points.len();
        assert!(point_index < size);

        let point = match points.get(point_index) {
            Some(point) => point,
            None => panic!(
                "failed to find angle: index is out of bounds for Tile {:?}",
                self
            ),
        };
        let point1 = match points.get((point_index + (size - 1)) % size) {
            Some(point) => point,
            None => panic!(
                "failed to find angle: preceding index is out of bounds for Tile {:?}",
                self
            ),
        };
        let point2 = match points.get((point_index + 1) % size) {
            Some(point) => point,
            None => panic!(
                "failed to find angle: succeeding index is out of bounds for Tile {:?}",
                self
            ),
        };
        let angle = Point::angle(point1, point, point2);
        rad(if self.parity { TAU - angle } else { angle })
    }

    // angles returns all angles of the tile
    pub fn angles(&self) -> Vec<f64> {
        (0..self.size()).map(|point_index| self.angle(point_index)).collect()
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
                    None => panic!("could not find point {} for Tile {:?}", i, self),
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
                "couldn't calc max actual side length for Prototile {:?}",
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
                    "couldn't calc max expected side length for Prototile {:?}",
                    self
                ),
            };

        for (exp, actual) in izip!(exp_side_lengths.iter(), actual_side_lengths.iter()) {
            approx_eq!(f64, exp / max_exp, actual / max_actual);
        }
    }

    // closest_edge finds the closest edge in the tile to the provided point
    pub fn closest_edge<'a>(&'a self, point: &Point) -> Edge<'a> {
        let edges = Point::edges(&self.points);
        let mut min = f64::MAX;
        let mut closest_edge = edges.get(0).unwrap();
        for edge in edges.iter() {
            let distance = edge.distance(point);
            if distance < min {
                min = distance;
                closest_edge = &edge;
            }
        }
        if !self.parity {
            Edge(closest_edge.0, closest_edge.1)
        } else {
            Edge(closest_edge.1, closest_edge.0)
        }
    }

    // contains determines whether or not the provided point is contained within the tile
    // https://alienryderflex.com/polygon
    pub fn contains(&self, point: &Point) -> bool {
        let edges = Point::edges(&self.points);
        let mut odd_nodes = false;
        for edge in edges.iter() {
            let start = edge.0;
            let stop = edge.1;
            if
                ((stop.1 < point.1 && start.1 >= point.1) || (start.1 < point.1 && stop.1 >= point.1)) &&
                (start.0 <= point.0 || stop.0 <= point.0)
            {
                odd_nodes ^= (stop.0 + (point.1 - stop.1) / (start.1 - stop.1) * (start.0 - stop.0)) < point.0;
            }
        }
        odd_nodes
    }

    pub fn edges<'a>(&'a self) -> Vec<Edge<'a>> {
        Point::edges(&self.points)
    }

    // reorient_about_origin shifts the underlying points of a Tile so that the first
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

    // size returns the number of points of the tile
    pub fn size(&self) -> usize {
        self.points.len()
    }

    // centroid computes the centroid of the provided points
    fn centroid(points: &Vec<Point>) -> Point {
        // calc area
        let terms = Point::edges(points)
            .into_iter()
            .map(|Edge(p0, p1)| {
                let conv = p0.0 * p1.1 - p0.1 * p1.0;
                (conv, conv * (p0.0 + p1.0), conv * (p0.1 + p1.1))
            })
            .reduce(|(a0, a1, a2), (e0, e1, e2)| (a0 + e0, a1 + e1, a2 + e2))
            .unwrap();
        let area = terms.0 / 2.;
        Point(terms.1 / (6. * area), terms.2 / (6. * area))
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

impl Spatial for Tile {
    type Hashed = Point;

    fn distance(&self, point: &Point) -> f64 {
        let edges = Point::edges(&self.points);

        // duplicate Tile::contains to avoid recreating the edges iterator twice
        let mut odd_nodes = false;
        for edge in edges.iter() {
            let start = edge.0;
            let stop = edge.1;
            if
                ((stop.1 < point.1 && start.1 >= point.1) || (start.1 < point.1 && stop.1 >= point.1)) &&
                (start.0 <= point.0 || stop.0 <= point.0)
            {
                odd_nodes ^= (stop.0 + (point.1 - stop.1) / (start.1 - stop.1) * (start.0 - stop.0)) < point.0;
            }
        }
        if odd_nodes {
            return 0.;
        }

        // closest edge
        let mut min_distance = f64::MAX;
        for edge in edges.iter() {
            let distance = edge.distance(point);
            if distance < min_distance {
                min_distance = distance;
            }
        }
        min_distance
    }

    fn intersects(&self, bounds: &Bounds) -> bool {
        Point::edges(&self.points).iter().any(|edge| edge.intersects(bounds))
    }

    fn key(&self) -> Point {
        self.centroid.clone()
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

pub fn regular_polygon(side_length: f64, num_sides: usize) -> Tile {
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

    let tile = Tile::new(
        iter::repeat(Point(0., 0.))
            .take(num_sides)
            .enumerate()
            .map(|(i, point)| point.transform(&generator(i)))
            .collect(),
    );

    tile.assert_angles(
        iter::repeat(2. * centroid_angle_of_inclination)
            .take(num_sides)
            .collect(),
    );
    tile.assert_sides(iter::repeat(side_length).take(num_sides).collect());

    tile
}

// star_polygon returns a Tile with 2 * num_base_sides points, where each point which
// was included in the original regular polygon has its internal angle set to the provided value.
pub fn star_polygon(side_length: f64, num_base_sides: usize, internal_angle: f64) -> Tile {
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

    Tile::new(dented_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{fmt_float, to_rad};
    use geometry::{ORIGIN, Point};

    const X: Point = Point(1., 0.);

    #[test]
    fn test_tile_closest_edge() {
        let square = regular_polygon(1., 4);

        let edge = square.closest_edge(&Point::new((0.5, -0.5)));
        assert_eq!(&Point(0., 0.), edge.0);
        assert_eq!(&Point(1., 0.), edge.1);

        let edge = square.closest_edge(&Point::new((1.5, 0.5)));
        assert_eq!(&Point(1., 0.), edge.0);
        assert_eq!(&Point(1., 1.), edge.1);

        let edge = square.closest_edge(&Point::new((0.5, 1.5)));
        assert_eq!(&Point(1., 1.), edge.0);
        assert_eq!(&Point(0., 1.), edge.1);

        let edge = square.closest_edge(&Point::new((-0.5, 0.5)));
        assert_eq!(&Point(0., 1.), edge.0);
        assert_eq!(&Point(0., 0.), edge.1);
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
