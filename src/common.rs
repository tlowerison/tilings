use float_cmp::ApproxEq;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    iter,
    ops::{Add,Neg,Sub},
};
use wasm_bindgen::prelude::*;

pub const APPROX_FLOAT_PRECISION_F: f64 = 1000.;
pub const APPROX_FLOAT_PRECISION_I: i32 = 1000;
pub const IDENTITY_MATRIX: [[f64; 2]; 2] = [[1.,0.],[0.,1.]];
pub const IDENTITY_VECTOR: [f64; 2] = [0.,0.];
pub const IDENTITY_AFFINE: Affine = Affine(IDENTITY_MATRIX, IDENTITY_VECTOR);
pub const ORIGIN: Point = Point(0.,0.);
pub const POINT_PRECISION: f64 = 1_000_000.; // TODO: Properly align this value with POINT_MARGIN
pub const POINT_MARGIN: (f64, i64) = (0.000_001, 5);

#[wasm_bindgen]
pub struct Point(pub(crate) f64, pub(crate) f64);

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

pub trait Transform {
    fn as_affine(&self) -> Affine;
}

pub trait Transformable<'a> {
    fn transform<T: Transform>(&self, transform: &'a T) -> Self;
}

pub trait DelayedTransformable<'a> {
    type Item;
    fn add<T: Transform>(&mut self, transform: &'a T) -> &Self;
    fn reduce(&self) -> Self::Item;
}

pub struct Affine(pub(crate) [[f64; 2]; 2], pub(crate) [f64; 2]); // (affine matrix, translation vector)

impl Affine {
    pub fn is_flip(&self) -> bool {
        self.0[0][0] * self.0[1][1] - self.0[0][1] * self.0[1][0] < 0.
    }

    fn mul_0(lhs: &Affine, rhs: &Affine, i: usize, j: usize) -> f64 {
        lhs.0[j][0] * rhs.0[0][i] + lhs.0[j][1] * rhs.0[1][i]
    }

    fn mul_1(lhs: &Affine, rhs: &Affine, i: usize) -> f64 {
        lhs.0[i][0] * rhs.1[0] + lhs.0[i][1] * rhs.1[1] + lhs.1[i]
    }
}

impl Transform for Affine {
    fn as_affine(&self) -> Affine {
        *self
    }
}

impl Copy for Affine {}

impl Clone for Affine {
    fn clone(&self) -> Affine {
        Affine(
            [[self.0[0][0], self.0[1][0]], [self.0[0][1], self.0[1][1]]],
            [self.1[0], self.1[1]],
        )
    }
}

impl<'a> Transformable<'a> for Affine {
    fn transform<T: Transform>(&self, transform: &'a T) -> Self {
        let lhs = &transform.as_affine();
        let rhs = self;
        Affine(
            [
                [Affine::mul_0(lhs, rhs, 0, 0), Affine::mul_0(lhs, rhs, 0, 1)],
                [Affine::mul_0(lhs, rhs, 1, 0), Affine::mul_0(lhs, rhs, 1, 1)],
            ],
            [
                Affine::mul_1(lhs, rhs, 0),
                Affine::mul_1(lhs, rhs, 1),
            ],
        )
    }
}

impl<'a> Transformable<'a> for Point {
    fn transform<T: Transform>(&self, transform: &'a T) -> Self {
        let lhs = transform.as_affine();
        let rhs = Affine(IDENTITY_MATRIX, [self.0, self.1]);
        Point(
            Affine::mul_1(&lhs, &rhs, 0),
            Affine::mul_1(&lhs, &rhs, 1),
        )
    }
}

// Euclidean group transformations
pub enum Euclid {
    Composite(Affine),
    Translate((f64, f64)), // parameterizes (dx,dy) to move an object by (i.e. underlying reference frame is not shifted)
    Rotate(f64), // parameterizes angle through the origin which an object will be rotated by - expects radians
    Flip(f64), // parameterizes angle through the origin of flip line - expects radians
}

impl Transform for Euclid {
    fn as_affine(&self) -> Affine {
        match self {
            Euclid::Composite(affine) => affine.clone(),
            Euclid::Translate((dx, dy)) => Affine(IDENTITY_MATRIX, [*dx, *dy]),
            Euclid::Rotate(radians) => {
                let radians = *radians;
                let cos = radians.cos();
                let sin = radians.sin();
                Affine([[cos, -sin], [sin, cos]], IDENTITY_VECTOR)
            },
            Euclid::Flip(radians) => {
                let radians = *radians;
                let cos = radians.cos();
                let sin = radians.sin();
                Affine([[cos, sin], [sin, -cos]], IDENTITY_VECTOR)
            },
        }
    }
}

impl<'a> Transformable<'a> for Euclid {
    fn transform<T: Transform>(&self, transform: &'a T) -> Self {
        let affine = &transform.as_affine();
        if let Euclid::Composite(t) = self {
            return Euclid::Composite(t.transform(affine))
        } else {
            return Euclid::Composite(self.as_affine().transform(affine))
        }
    }
}

pub struct Generator {
    affine: Affine,
    generated: Vec<Affine>,
}

impl Generator {
    pub fn new(affine: Affine) -> impl FnMut(usize) -> Affine {
        let mut generator = Generator {
            affine,
            generated: vec![IDENTITY_AFFINE, affine],
        };
        move |n| {
            if n < generator.generated.len() {
                return generator.generated.get(n).unwrap().clone()
            }
            let mut new_affine = generator.generated.last().unwrap().clone();
            for i in generator.generated.len()-1..n {
                new_affine = generator.affine.transform(generator.generated.last().unwrap());
                generator.generated.push(new_affine.clone());
            }
            new_affine
        }
    }
}

// approx_f64 multiplies f by a power of 10 then cuts off all fractional digits by rounding
pub fn approx_f64(f: f64) -> i32 {
    (f * APPROX_FLOAT_PRECISION_F).round() as i32
}

pub fn calc_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub fn fmt_f64<'a>(f: f64) -> String {
    let i = approx_f64(f);
    format!("{}{}.{}", if i < 0 { "-" } else { "" }, (i / APPROX_FLOAT_PRECISION_I).abs(), (i % APPROX_FLOAT_PRECISION_I).abs())
}

// reduce_transforms compresses a sequence of transforms into a single affine
// such that the output transform is equivalent to applying the transforms from left to right
// Ex: x.transform(&reduce_transforms(vec![A, B, C])) =~ x.transform(&A).transform(&B).transform(&C), or in matrix notation, =~ C * B * A * x
pub fn reduce_transforms<T: Transform>(transforms: &Vec<T>) -> Affine {
    match iter::once(IDENTITY_AFFINE).chain(transforms.into_iter().map(|t| t.as_affine())).reduce(|a,e| a.transform(&e)) {
        Some(affine) => affine,
        None => panic!("unable to reduce transforms"),
    }
}

pub fn range_iter(range: std::ops::Range<usize>, rev: bool) -> itertools::Either<impl Iterator<Item = usize>, impl Iterator<Item = usize>> {
    if !rev {
        itertools::Either::Left(range)
    } else {
        itertools::Either::Right(range.rev())
    }
}

// Display

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{}]", fmt_f64(self.0), fmt_f64(self.1))
    }
}

impl std::fmt::Display for Euclid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Euclid::Composite(affine) => write!(f, "{}", affine),
            Euclid::Translate((dx, dy)) => write!(f, "T({}, {})", *dx, *dy),
            Euclid::Rotate(revolutions) => write!(f, "R({}π)", 2.0 * (*revolutions)),
            Euclid::Flip(revolutions) => write!(f, "F({}π)", 2.0 * (*revolutions)),
        }
    }
}

fn dupe_str(s: &str, n: u32) -> String {
    (0..n).map(|_| s).collect::<String>()
}

fn str_digits(s: &str) -> (u32, u32) {
    let chunks: Vec<&str> = s.split(".").collect();
    let trunc_digits = match chunks.get(0) { Some(num) => num.len() as u32, None => panic!("digits_str: failed to split string") };
    let fract_digits = match chunks.get(1) { Some(num) => (num.len() + 1) as u32, None => 0 };
    (trunc_digits, fract_digits)
}

fn calc_max_digits(vals: &[&str]) -> (u32, u32) {
    match vals.iter()
        .map(|val| str_digits(val))
        .reduce(|max_digits, digits| (max_digits.0.max(digits.0), max_digits.1.max(digits.1)))
    { Some(opt) => opt, None => (0, 0) }
}

fn wrap_f64(f: &str, (max_trunc_digits, max_fract_digits): (u32, u32)) -> String {
    let (trunc_digits, fract_digits) = str_digits(f);
    format!("{}{}{}", dupe_str(" ", max_trunc_digits - trunc_digits), f, dupe_str(" ", max_fract_digits - fract_digits))
}

impl std::fmt::Display for Affine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
