use crate::{
    bounds::{Bounds, Spatial},
    point::{ORIGIN, Point},
};
use common::DEFAULT_F64_MARGIN;
use float_cmp::ApproxEq;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub struct Edge<'a>(pub &'a Point, pub &'a Point);

impl<'a> Hash for Edge<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl<'a> Spatial for Edge<'a> {
    type Hashed = Edge<'a>;

    fn distance(&self, point: &Point) -> f64 {
        let edge = self.1 - self.0;
        let t = ((point - self.0).dot(&edge) / edge.norm_squared()).clamp(0., 1.);
        let projection = self.0 + &edge.mul(t);
        (point - &projection).norm()
    }

    // https://stackoverflow.com/questions/4977491/determining-if-two-line-segments-intersect/4977569#4977569
    fn intersects(&self, bounds: &Bounds) -> bool {
        if self.0.intersects(bounds) && self.1.intersects(bounds) {
            return true
        }

        let points = vec![
            &bounds.center + &Point(bounds.radius, bounds.radius),
            &bounds.center + &Point(-bounds.radius, bounds.radius),
            &bounds.center + &Point(-bounds.radius, -bounds.radius),
            &bounds.center + &Point(bounds.radius, -bounds.radius),
        ];

        let edges = Point::edges(&points);

        let u0 = self.0;
        let v0 = self.1 - u0;

        for edge in edges.into_iter() {
            let u1 = edge.0;
            let w = u0 - u1;
            if w == ORIGIN {
                return true
            }

            let v1 = edge.1 - u1;
            let determinant = v1.0 * v0.1 - v0.0 * v1.1;

            if !0_f64.approx_eq(determinant, DEFAULT_F64_MARGIN) {
                let s = w.dot(&Point(v0.1, -v0.0)) / determinant;
                let t = w.dot(&Point(v1.1, -v1.0)) / determinant;

                if s >= 0. && s <= 1. && t >= 0. && t <= 1. {
                    return true
                }
            }
        }
        return false
    }

    fn key(&self) -> Self::Hashed {
        self.clone()
    }
}
