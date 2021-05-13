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

const DISPLAY_PRECISION: u32 = 2;

// POINT_MARGIN guarantees that a pair of points with neither coordinate
// differing by more than POINT_MARGIN.0 / 10. will hash to the same value.
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
        ((self.0 / POINT_MARGIN.0).round() as i32).hash(state);
        ((self.1 / POINT_MARGIN.0).round() as i32).hash(state);
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
        write!(
            f,
            "[{},{}]",
            fmt_float::<f64>(NumCast::from(self.0).unwrap(), DISPLAY_PRECISION),
            fmt_float::<f64>(NumCast::from(self.1).unwrap(), DISPLAY_PRECISION)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::euclid::*;
    use common::calc_hash;
    use float_cmp::*;
    use std::f64::consts::{PI, TAU};

    #[test]
    fn test_point_new() {
        let values = (PI, TAU);
        let point = Point::new(values);
        approx_eq!(f64, values.0, point.0);
        approx_eq!(f64, values.1, point.1);
    }

    #[test]
    fn test_point_arg() {
        approx_eq!(f64, 0. * PI / 4., Point(1., 0.).arg());
        approx_eq!(f64, 1. * PI / 4., Point(1., 1.).arg());
        approx_eq!(f64, 2. * PI / 4., Point(0., 1.).arg());
        approx_eq!(f64, 3. * PI / 4., Point(-1., 1.).arg());
        approx_eq!(f64, 4. * PI / 4., Point(-1., 0.).arg());
        approx_eq!(f64, 5. * PI / 4., Point(-1., -1.).arg());
        approx_eq!(f64, 6. * PI / 4., Point(0., -1.).arg());
        approx_eq!(f64, 7. * PI / 4., Point(1., -1.).arg());
    }

    #[test]
    fn test_point_neg() {
        let point = Point(1., 1.);
        approx_eq!(f64, -1., (-point).0);
        approx_eq!(f64, -1., (-point).1);
        approx_eq!(f64, -1., point.neg().0);
        approx_eq!(f64, -1., point.neg().1);
    }

    #[test]
    fn test_point_dot() {
        approx_eq!(f64, 36., Point(3., 4.).dot(&Point(-2., 10.)));
    }

    #[test]
    fn test_point_norm() {
        approx_eq!(f64, 5., Point(3., 4.).norm());
    }

    #[test]
    fn test_point_values() {
        let values = Point(1., 2.).values();
        approx_eq!(f64, 1., values.0);
        approx_eq!(f64, 2., values.1);
    }

    #[test]
    fn test_point_add() {
        let point = &Point(1., 2.) + &Point(-2., 3.);
        approx_eq!(f64, -1., point.0);
        approx_eq!(f64, 5., point.1);
    }

    #[test]
    fn test_point_clone() {
        let point = Point(3., 5.);
        let point_clone = point.clone();
        approx_eq!(f64, point.0, point_clone.0);
        approx_eq!(f64, point.1, point_clone.1);
    }

    #[test]
    fn test_point_hash() {
        let point0 = Point(PI, TAU);
        let point1 = Point(PI - POINT_MARGIN.0, TAU + POINT_MARGIN.0);
        assert_ne!(calc_hash(&point0), calc_hash(&point1));

        let point0 = Point(PI, TAU);
        let point1 = Point(PI - POINT_MARGIN.0 / 10., TAU + POINT_MARGIN.0 / 10.);
        assert_eq!(calc_hash(&point0), calc_hash(&point1));
    }

    #[test]
    fn test_point_eq() {
        let point0 = Point(PI, TAU);
        let point1 = Point(PI - POINT_MARGIN.0, TAU + POINT_MARGIN.0);
        assert!(point0 != point1);

        let point0 = Point(PI, TAU);
        let point1 = Point(PI - POINT_MARGIN.0 / 10., TAU + POINT_MARGIN.0 / 10.);
        assert!(point0 == point1);
    }

    #[test]
    fn test_point_sub() {
        let point = &Point(1., 2.) + &Point(-2., 3.);
        approx_eq!(f64, 3., point.0);
        approx_eq!(f64, -1., point.1);
    }

    #[test]
    fn test_point_transform() {
        let point = Point(1., 1.);
        let new_point = point.transform(&Euclid::Translate((-1., 1.)));
        approx_eq!(f64, 0., new_point.0);
        approx_eq!(f64, 2., new_point.1);

        let point = Point(1., 1.);
        let new_point = point.transform(&Euclid::Rotate(PI / 4.));
        approx_eq!(f64, 0., new_point.0);
        approx_eq!(f64, 2_f64.sqrt(), new_point.1);

        let point = Point(-2., 1.);
        let new_point = point.transform(&Euclid::Flip(0.));
        approx_eq!(f64, -2., new_point.0);
        approx_eq!(f64, -1., new_point.1);

        let point = Point(-2., 1.);
        let new_point = point.transform(&Euclid::Flip(PI / 2.));
        approx_eq!(f64, 2., new_point.0);
        approx_eq!(f64, 1., new_point.1);

        let point = Point(-2., 1.);
        let new_point = point.transform(&Euclid::Flip(PI / 4.));
        approx_eq!(f64, 1., new_point.0);
        approx_eq!(f64, -2., new_point.1);

        let point = Point(3.1, 4.2);
        let new_point = point.transform(&Affine([[2., 4.], [-3., 5.]], [-1., 1.]));
        approx_eq!(f64, 13.6, new_point.0);
        approx_eq!(f64, 12.7, new_point.0);
    }

    #[test]
    fn test_point_fmt() {
        assert_eq!("[0.00,0.00]", format!("{}", Point(0., 0.)));
        assert_eq!("[1.45,-1.45]", format!("{}", Point(1.449, -1.449)));
        assert_eq!("[2.00,-2.00]", format!("{}", Point(1.999, -1.999)));
    }
}
