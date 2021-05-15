use crate::transform::{Transform, Transformable};
use common::*;
use num_traits::cast::NumCast;
use std::iter;

pub const IDENTITY_AFFINE: Affine = Affine([[1., 0.], [0., 1.]], [0., 0.]);
const DISPLAY_PRECISION: u32 = 3;

pub struct Affine(pub [[f64; 2]; 2], pub [f64; 2]); // (row-major transform matrix, translation vector)

impl Affine {
    pub fn is_flip(&self) -> bool {
        self.0[0][0] * self.0[1][1] - self.0[0][1] * self.0[1][0] < 0.
    }

    pub(crate) fn mul_0(lhs: &Affine, rhs: &Affine, i: usize, j: usize) -> f64 {
        lhs.0[j][0] * rhs.0[0][i] + lhs.0[j][1] * rhs.0[1][i]
    }

    pub(crate) fn mul_1(lhs: &Affine, rhs: &Affine, i: usize) -> f64 {
        lhs.0[i][0] * rhs.1[0] + lhs.0[i][1] * rhs.1[1] + lhs.1[i]
    }
}

impl Clone for Affine {
    fn clone(&self) -> Affine {
        Affine(
            [[self.0[0][0], self.0[1][0]], [self.0[0][1], self.0[1][1]]],
            [self.1[0], self.1[1]],
        )
    }
}

impl Copy for Affine {}

impl Transform for Affine {
    fn as_affine(&self) -> Affine {
        *self
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
            [Affine::mul_1(lhs, rhs, 0), Affine::mul_1(lhs, rhs, 1)],
        )
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
                return generator.generated.get(n).unwrap().clone();
            }
            let mut new_affine = generator.generated.last().unwrap().clone();
            for _ in generator.generated.len() - 1..n {
                new_affine = generator
                    .affine
                    .transform(generator.generated.last().unwrap());
                generator.generated.push(new_affine.clone());
            }
            new_affine
        }
    }
}

impl std::fmt::Display for Affine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vals: [[String; 3]; 3] = [
            [
                fmt_float::<f64>(NumCast::from(self.0[0][0]).unwrap(), DISPLAY_PRECISION),
                fmt_float::<f64>(NumCast::from(self.0[0][1]).unwrap(), DISPLAY_PRECISION),
                fmt_float::<f64>(NumCast::from(self.1[0]).unwrap(), DISPLAY_PRECISION),
            ],
            [
                fmt_float::<f64>(NumCast::from(self.0[1][0]).unwrap(), DISPLAY_PRECISION),
                fmt_float::<f64>(NumCast::from(self.0[1][1]).unwrap(), DISPLAY_PRECISION),
                fmt_float::<f64>(NumCast::from(self.1[1]).unwrap(), DISPLAY_PRECISION),
            ],
            [fmt_float(0., DISPLAY_PRECISION), fmt_float(0., DISPLAY_PRECISION), fmt_float(1., DISPLAY_PRECISION)],
        ];
        let max_digits: [(u32, u32); 3] = [
            calc_max_digits(&[&vals[0][0], &vals[1][0], &vals[2][0]]),
            calc_max_digits(&[&vals[0][1], &vals[1][1], &vals[2][1]]),
            calc_max_digits(&[&vals[0][2], &vals[1][2], &vals[2][2]]),
        ];

        let mut display = String::new();
        for i in 0..3 {
            display.push_str(
                format!(
                    "[{} {} {}]\n",
                    wrap_f64(&vals[i][0], max_digits[0]),
                    wrap_f64(&vals[i][1], max_digits[1]),
                    wrap_f64(&vals[i][2], max_digits[2]),
                )
                .as_str(),
            );
        }

        write!(f, "{}", display.trim_end())
    }
}

fn str_digits(s: &str) -> (u32, u32) {
    let chunks: Vec<&str> = s.split(".").collect();
    let trunc_digits = match chunks.get(0) {
        Some(num) => num.len() as u32,
        None => panic!("digits_str: failed to split string"),
    };
    let fract_digits = match chunks.get(1) {
        Some(num) => (num.len() + 1) as u32,
        None => 0,
    };
    (trunc_digits, fract_digits)
}

fn calc_max_digits(vals: &[&str]) -> (u32, u32) {
    match vals
        .iter()
        .map(|val| str_digits(val))
        .reduce(|max_digits, digits| (max_digits.0.max(digits.0), max_digits.1.max(digits.1)))
    {
        Some(opt) => opt,
        None => (0, 0),
    }
}

fn wrap_f64(f: &str, (max_trunc_digits, max_fract_digits): (u32, u32)) -> String {
    let (trunc_digits, fract_digits) = str_digits(f);
    format!(
        "{}{}{}",
        iter::repeat(String::from(" "))
            .take((max_trunc_digits - trunc_digits) as usize)
            .collect::<Vec<String>>()
            .join(""),
        f,
        iter::repeat(String::from(" "))
            .take((max_fract_digits - fract_digits) as usize)
            .collect::<Vec<String>>()
            .join(""),
    )
}
