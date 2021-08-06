use crate::point::Point;
use std::hash::{Hash, Hasher};

pub const E: Point  = Point(0.5, 0.);
pub const NE: Point = Point(0.5, 0.5);
pub const N: Point  = Point(0., 0.5);
pub const NW: Point = Point(-0.5, 0.5);
pub const W: Point  = Point(-0.5, 0.);
pub const SW: Point = Point(-0.5, -0.5);
pub const S: Point  = Point(0., -0.5);
pub const SE: Point = Point(0.5, -0.5);

#[derive(Clone, Debug)]
pub struct Bounds {
    pub center: Point,
    pub radius: f64, // length from center to middle of edge
}

#[derive(Debug)]
pub struct SplitBounds {
    pub ne: Bounds,
    pub nw: Bounds,
    pub se: Bounds,
    pub sw: Bounds,
}

pub trait Spatial {
    type Hashed;
    fn distance(&self, point: &Point) -> f64;
    fn intersects(&self, bounds: &Bounds) -> bool;
    fn key(&self) -> Self::Hashed;
}

impl Bounds {
    pub fn mul(&self, coefficient: f64) -> Bounds {
        Bounds {
            center: self.center,
            radius: self.radius * coefficient,
        }
    }

    // will return 0 if point is inside bounds
    pub fn distance_vector(&self, point: &Point) -> Point {
        Point(
            0_f64.max((point.0 - self.center.0).abs() - self.radius),
            0_f64.max((point.1 - self.center.1).abs() - self.radius),
        )
    }

    pub fn shift(&self, offset: &Point) -> Bounds {
        Bounds {
            radius: self.radius,
            center: &self.center + offset,
        }
    }

    pub fn split(&self) -> SplitBounds {
        let radius = self.radius / 2.;
        SplitBounds {
            ne: Bounds { radius, center: &self.center + &NE.mul(self.radius) },
            nw: Bounds { radius, center: &self.center + &NW.mul(self.radius) },
            se: Bounds { radius, center: &self.center + &SE.mul(self.radius) },
            sw: Bounds { radius, center: &self.center + &SW.mul(self.radius) },
        }
    }
}

impl Eq for Bounds {}

// Bounds is Hashed by its center point ~only~. Bounds with different radii but the same center
// will hash to the same thing. This is expected and optimized since bounds are expected to never
// have intersecting centers in this workspace's use cases.
impl Hash for Bounds {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.center.hash(state);
    }
}

impl PartialEq for Bounds {
    fn eq(&self, other: &Self) -> bool {
        self.center.eq(&other.center)
    }
}

impl Spatial for Bounds {
    type Hashed = Point;
    // https://gamedev.stackexchange.com/questions/44483/how-do-i-calculate-distance-between-a-point-and-an-axis-aligned-rectangle
    fn distance(&self, point: &Point) -> f64 {
        self.distance_vector(point).norm()
    }

    // https://stackoverflow.com/questions/20925818/algorithm-to-check-if-two-boxes-overlap
    fn intersects(&self, bounds: &Bounds) -> bool {
        (self.center.0 - self.radius < bounds.center.0 + bounds.radius) &&
        (self.center.0 + self.radius > bounds.center.0 - bounds.radius) &&
        (self.center.1 - self.radius < bounds.center.1 + bounds.radius) &&
        (self.center.1 + self.radius > bounds.center.1 - bounds.radius)
    }

    fn key(&self) -> Self::Hashed {
        self.center.clone()
    }
}
