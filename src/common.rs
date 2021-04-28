use core::fmt;
use std::{
    f64::consts::TAU,
    ops::Mul,
};

pub struct Coord(pub(crate) f64, pub(crate) f64);

pub struct Transform(pub(crate) [[f64; 2]; 2], pub(crate) [f64; 2]); // (transform matrix, translation vector)

// Euclidean group transformations
pub enum Euclid {
    Translate(f64, f64), // parameterizes (dx,dy) to move an object by (i.e. underlying reference frame is not shifted)
    Rotate(f64), // parameterizes angle through the origin which an object will be rotated by - use revolutions i.e. radians / 2π
    Flip(f64), // parameterizes angle through the origin of flip line
    Identity,
}

impl Mul<Transform> for Transform {
    type Output = Transform;
    fn mul(self, rhs: Transform) -> Transform {
        let lhs_m0 = self.0[0];
        let lhs_m1 = self.0[1];
        let rhs_m0 = rhs.0[0];
        let rhs_m1 = rhs.0[1];
        Transform(
            [
                [lhs_m0[0] * rhs_m0[0] + lhs_m0[1] * rhs_m1[0], lhs_m0[0] * rhs_m0[1] + lhs_m0[1] * rhs_m1[1]],
                [lhs_m1[0] * rhs_m0[0] + lhs_m1[1] * rhs_m1[0], lhs_m1[0] * rhs_m0[1] + lhs_m1[1] * rhs_m1[1]],
            ],
            [
                lhs_m0[0] * rhs.1[0] + lhs_m0[1] * rhs.1[1] + self.1[0],
                lhs_m1[0] * rhs.1[0] + lhs_m1[1] * rhs.1[1] + self.1[1],
            ],
        )
    }
}

impl Mul<Coord> for Transform {
    type Output = Coord;
    fn mul(self, rhs: Coord) -> Coord {
        Coord(
            self.0[0][0] * rhs.0 + self.0[0][1] * rhs.1 + self.1[0],
            self.0[1][0] * rhs.0 + self.0[1][1] * rhs.1 + self.1[1],
        )
    }
}

impl Euclid {
    fn as_transform(&self) -> Transform {
        match *self {
            Euclid::Translate(dx, dy) => Transform(IDENTITY_MATRIX, [dx, dy]),
            Euclid::Rotate(revolutions) => {
                let radians = TAU * revolutions;
                let cos = radians.cos();
                let sin = radians.sin();
                Transform([[cos, -sin], [sin, cos]], IDENTITY_VECTOR)
            },
            Euclid::Flip(revolutions) => {
                let radians = 2.0 * TAU * revolutions;
                let cos = radians.cos();
                let sin = radians.sin();
                Transform([[cos, sin], [sin, -cos]], IDENTITY_VECTOR)
            },
            Euclid::Identity => IDENTITY_TRANSFORM,
        }
    }
}

pub const IDENTITY_MATRIX: [[f64; 2]; 2] = [[1.,0.],[0.,1.]];
pub const IDENTITY_VECTOR: [f64; 2] = [0.,0.];
pub const IDENTITY_TRANSFORM: Transform = Transform(IDENTITY_MATRIX, IDENTITY_VECTOR);

pub fn compose_euclids(euclids: &[&Euclid]) -> Transform {
    if euclids.len() == 0 {
        return IDENTITY_TRANSFORM
    } else if euclids.len() == 1 {
        return euclids[0].as_transform()
    }
    let mut transform = euclids[0].as_transform();
    for euclid in euclids[1..].into_iter() {
        transform = transform * euclid.as_transform()
    }
    return transform
}

// Display

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", fmt_f64(self.0), fmt_f64(self.1))
    }
}

impl fmt::Display for Euclid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Euclid::Translate(dx, dy) => write!(f, "T({}, {})", dx, dy),
            Euclid::Rotate(revolutions) => write!(f, "R({}π)", 2.0 * revolutions),
            Euclid::Flip(revolutions) => write!(f, "F({}π)", 2.0 * revolutions),
            Euclid::Identity => write!(f, "I"),
        }
    }
}

fn dupe_str(s: &str, n: u32) -> String {
    (0..n).map(|_| s).collect::<String>()
}

fn fmt_f64<'a>(f: f64) -> String {
    format!("{}", ((10000. * f).trunc() / 10.).round() / 1000.)
}

fn str_digits(s: &str) -> (u32, u32) {
    let chunks: Vec<&str> = s.split(".").collect();
    let trunc_digits = match chunks.get(0) { Some(num) => num.len() as u32, None => panic!("digits_str: failed to split string") };
    let fract_digits = match chunks.get(1) { Some(num) => (num.len() + 1) as u32, None => 0 };
    (trunc_digits, fract_digits)
}

fn calc_max_digits(vals: &[&str]) -> (u32, u32) {
    match vals.into_iter()
        .map(|val| str_digits(val))
        .reduce(|max_digits, digits| (max_digits.0.max(digits.0), max_digits.1.max(digits.1)))
    { Some(opt) => opt, None => (0, 0) }
}

fn wrap_f64(f: &str, (max_trunc_digits, max_fract_digits): (u32, u32)) -> String {
    let (trunc_digits, fract_digits) = str_digits(f);
    format!("{}{}{}", dupe_str(" ", max_trunc_digits - trunc_digits), f, dupe_str(" ", max_fract_digits - fract_digits))
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vals: [[String; 3]; 3] = [
            [fmt_f64(self.0[0][0]), fmt_f64(self.0[0][1]), fmt_f64(self.1[0])],
            [fmt_f64(self.0[1][0]), fmt_f64(self.0[1][1]), fmt_f64(self.1[1])],
            [String::from("0"), String::from("0"), String::from("1")],
        ];
        let max_digits: [(u32, u32); 3] = [
            calc_max_digits(&[&vals[0][0], &vals[1][0], &vals[2][0]]),
            calc_max_digits(&[&vals[0][1], &vals[1][1], &vals[2][1]]),
            calc_max_digits(&[&vals[0][2], &vals[1][2], &vals[2][2]]),
        ];

        let mut display = String::new();
        for i in 0..3 {
            display.push_str(format!("[{} {} {}]\n",
                wrap_f64(&vals[i][0], max_digits[0]),
                wrap_f64(&vals[i][1], max_digits[1]),
                wrap_f64(&vals[i][2], max_digits[2]),
            ).as_str());
        }

        write!(f, "{}", display.trim_end())
    }
}
