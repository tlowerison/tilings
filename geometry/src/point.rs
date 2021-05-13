use crate::{
    affine::{Affine, IDENTITY_AFFINE},
    transform::{Transform, Transformable},
};

use common::*;
use float_cmp::ApproxEq;
use num_traits::cast::NumCast;
use std::{
    hash::{Hash, Hasher},
    ops::{Add, Neg, Sub},
};

pub const ORIGIN: Point = Point(0., 0.);
const DISPLAY_PRECISION: u32 = 3;
const POINT_PRECISION: f64 = 1_000_000.; // TODO: Properly align this value with POINT_MARGIN
const POINT_MARGIN: (f64, i64) = (0.000_001, 5);

pub struct Point(pub f64, pub f64);

impl Point {
    pub fn new(values: (f64, f64)) -> Point {
        Point(values.0, values.1)
    }

    pub fn arg(&self) -> f64 {
        self.1.atan2(self.0)
    }

    pub fn neg(&self) -> Point {
        Point(-self.0, -self.1)
    }

    pub fn dot(&self, other: &Point) -> f64 {
        self.0 * other.0 + self.1 * other.1
    }

    pub fn norm(&self) -> f64 {
        (self.0.powi(2) + self.1.powi(2)).sqrt()
    }

    pub fn values(&self) -> (f64, f64) {
        (self.0, self.1)
    }
}

impl Add for &Point {
    type Output = Point;
    fn add(self, other: &Point) -> Self::Output {
        Point(self.0 + other.0, self.1 + other.1)
    }
}

impl Clone for Point {
    fn clone(&self) -> Self {
        Point(self.0, self.1)
    }
}

impl Copy for Point {}

impl Eq for Point {}

impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ((self.0 * POINT_PRECISION).round() as i32).hash(state);
        ((self.1 * POINT_PRECISION).round() as i32).hash(state);
    }
}

impl Neg for Point {
    type Output = Point;
    fn neg(self) -> Self::Output {
        Point(-self.0, -self.1)
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.0.approx_eq(other.0, POINT_MARGIN) && self.1.approx_eq(other.1, POINT_MARGIN)
    }
}

impl Sub for &Point {
    type Output = Point;
    fn sub(self, other: &Point) -> Self::Output {
        Point(self.0 - other.0, self.1 - other.1)
    }
}

impl<'a> Transformable<'a> for Point {
    fn transform<T: Transform>(&self, transform: &'a T) -> Self {
        let lhs = transform.as_affine();
        let rhs = Affine(IDENTITY_AFFINE.0, [self.0, self.1]);
        Point(Affine::mul_1(&lhs, &rhs, 0), Affine::mul_1(&lhs, &rhs, 1))
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{}]", fmt_float::<f64>(NumCast::from(self.0).unwrap(), DISPLAY_PRECISION), fmt_float::<f64>(NumCast::from(self.1).unwrap(), DISPLAY_PRECISION))
    }
}
