use crate::transform::{Transform, Transformable};
use common::fmt_float;
use num_traits::cast::NumCast;
use std::iter;

pub const IDENTITY_AFFINE: Affine = Affine([[1., 0.], [0., 1.]], [0., 0.]);
const DISPLAY_PRECISION: u32 = 3;

pub struct Affine(pub [[f64; 2]; 2], pub [f64; 2]); // (row-major transform matrix, translation vector)

impl Affine {
    pub fn is_flip(&self) -> bool {
        self.0[0][0] * self.0[1][1] - self.0[0][1] * self.0[1][0] < 0.
    }

    // mul_0 computes the dot product between the ith row vector of lhs.0 and the jth column vector of rhs.0
    pub(crate) fn mul_0(lhs: &Affine, rhs: &Affine, i: usize, j: usize) -> f64 {
        lhs.0[i][0] * rhs.0[0][j] + lhs.0[i][1] * rhs.0[1][j]
    }

    // mul_1 computes the dot product between the ith row vector of lhs.0 and rhs.1
    pub(crate) fn mul_1(lhs: &Affine, rhs: &Affine, i: usize) -> f64 {
        lhs.0[i][0] * rhs.1[0] + lhs.0[i][1] * rhs.1[1] + lhs.1[i]
    }
}

impl Clone for Affine {
    fn clone(&self) -> Affine {
        Affine(
            [[self.0[0][0], self.0[0][1]], [self.0[1][0], self.0[1][1]]],
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
            [
                fmt_float(0., DISPLAY_PRECISION),
                fmt_float(0., DISPLAY_PRECISION),
                fmt_float(1., DISPLAY_PRECISION),
            ],
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::euclid::*;
    use common::approx_eq;
    use std::f64::consts::PI;

    #[test]
    fn test_affine_is_flip() {
        assert_eq!(true, Euclid::Flip(0.).as_affine().is_flip());
        assert_eq!(true, Euclid::Flip(1.).as_affine().is_flip());

        assert_eq!(false, Euclid::Rotate(1.).as_affine().is_flip());
        assert_eq!(false, Euclid::Translate((1., 1.)).as_affine().is_flip());

        assert_eq!(
            false,
            Euclid::Flip(0.)
                .transform(&Euclid::Flip(1.))
                .as_affine()
                .is_flip()
        );

        assert_eq!(
            true,
            Euclid::Rotate(1.)
                .transform(&Euclid::Flip(1.))
                .transform(&Euclid::Rotate(-1.))
                .as_affine()
                .is_flip()
        );
    }

    #[test]
    fn test_affine_mul_0() {
        let affine0 = Affine([[1., 2.], [3., 4.]], [5., 6.]);
        let affine1 = Affine([[1., 6.], [2., 5.]], [3., 4.]);

        // https://www.wolframalpha.com/input/?i=%7B%7B1%2C2%2C5%7D%2C%7B3%2C4%2C6%7D%2C%7B0%2C0%2C1%7D%7D+*+%7B%7B1%2C6%2C3%7D%2C%7B2%2C5%2C4%7D%2C%7B0%2C0%2C1%7D%7D
        approx_eq!(f64, 5., Affine::mul_0(&affine0, &affine1, 0, 0));
        approx_eq!(f64, 16., Affine::mul_0(&affine0, &affine1, 0, 1));
        approx_eq!(f64, 11., Affine::mul_0(&affine0, &affine1, 1, 0));
        approx_eq!(f64, 38., Affine::mul_0(&affine0, &affine1, 1, 1));

        // https://www.wolframalpha.com/input/?i=%7B%7B1%2C6%2C3%7D%2C%7B2%2C5%2C4%7D%2C%7B0%2C0%2C1%7D%7D+*+%7B%7B1%2C2%2C5%7D%2C%7B3%2C4%2C6%7D%2C%7B0%2C0%2C1%7D%7D
        approx_eq!(f64, 19., Affine::mul_0(&affine1, &affine0, 0, 0));
        approx_eq!(f64, 26., Affine::mul_0(&affine1, &affine0, 0, 1));
        approx_eq!(f64, 17., Affine::mul_0(&affine1, &affine0, 1, 0));
        approx_eq!(f64, 24., Affine::mul_0(&affine1, &affine0, 1, 1));
    }

    #[test]
    fn test_affine_mul_1() {
        let affine0 = Affine([[1., 2.], [3., 4.]], [5., 6.]);
        let affine1 = Affine([[1., 6.], [2., 5.]], [3., 4.]);

        // https://www.wolframalpha.com/input/?i=%7B%7B1%2C2%2C5%7D%2C%7B3%2C4%2C6%7D%2C%7B0%2C0%2C1%7D%7D+*+%7B%7B1%2C6%2C3%7D%2C%7B2%2C5%2C4%7D%2C%7B0%2C0%2C1%7D%7D
        approx_eq!(f64, 16., Affine::mul_1(&affine0, &affine1, 0));
        approx_eq!(f64, 31., Affine::mul_1(&affine0, &affine1, 1));

        // https://www.wolframalpha.com/input/?i=%7B%7B1%2C6%2C3%7D%2C%7B2%2C5%2C4%7D%2C%7B0%2C0%2C1%7D%7D+*+%7B%7B1%2C2%2C5%7D%2C%7B3%2C4%2C6%7D%2C%7B0%2C0%2C1%7D%7D
        approx_eq!(f64, 44., Affine::mul_1(&affine1, &affine0, 0));
        approx_eq!(f64, 44., Affine::mul_1(&affine1, &affine0, 1));
    }

    #[test]
    fn test_affine_clone() {
        let affine = Affine([[1., 2.], [3., 4.]], [5., 6.]);
        let affine_clone = affine.clone();

        approx_eq!(f64, affine_clone.0[0][0], affine.0[0][0]);
        approx_eq!(f64, affine_clone.0[0][1], affine.0[0][1]);
        approx_eq!(f64, affine_clone.0[1][0], affine.0[1][0]);
        approx_eq!(f64, affine_clone.0[1][1], affine.0[1][1]);
        approx_eq!(f64, affine_clone.1[0], affine.1[0]);
        approx_eq!(f64, affine_clone.1[1], affine.1[1]);
    }

    #[test]
    fn test_affine_as_affine() {
        let affine = Affine([[1., 2.], [3., 4.]], [5., 6.]);
        let affine_clone = affine.as_affine();

        approx_eq!(f64, affine_clone.0[0][0], affine.0[0][0]);
        approx_eq!(f64, affine_clone.0[0][1], affine.0[0][1]);
        approx_eq!(f64, affine_clone.0[1][0], affine.0[1][0]);
        approx_eq!(f64, affine_clone.0[1][1], affine.0[1][1]);
        approx_eq!(f64, affine_clone.1[0], affine.1[0]);
        approx_eq!(f64, affine_clone.1[1], affine.1[1]);
    }

    #[test]
    fn test_affine_transform() {
        let affine0 = Affine([[2., 5.], [0., -2.]], [11., 3.]);
        let affine1 = Affine([[4., -9.], [-1., PI]], [0., -8.]);

        // https://www.wolframalpha.com/input/?i=%7B%7B2%2C5%2C11%7D%2C%7B0%2C-2%2C3%7D%2C%7B0%2C0%2C1%7D%7D+*+%7B%7B4%2C-9%2C0%7D%2C%7B-1%2Cpi%2C-8%7D%2C%7B0%2C0%2C1%7D%7D
        let transformed = affine1.transform(&affine0);
        approx_eq!(f64, 3., transformed.0[0][0]);
        approx_eq!(f64, 5. * PI - 18., transformed.0[0][1]);
        approx_eq!(f64, 2., transformed.0[1][0]);
        approx_eq!(f64, -2. * PI, transformed.0[1][1]);
        approx_eq!(f64, -29., transformed.1[0]);
        approx_eq!(f64, 19., transformed.1[1]);

        // https://www.wolframalpha.com/input/?i=%7B%7B4%2C-9%2C0%7D%2C%7B-1%2Cpi%2C-8%7D%2C%7B0%2C0%2C1%7D%7D+*+%7B%7B2%2C5%2C11%7D%2C%7B0%2C-2%2C3%7D%2C%7B0%2C0%2C1%7D%7D
        let transformed = affine0.transform(&affine1);
        approx_eq!(f64, 8., transformed.0[0][0]);
        approx_eq!(f64, 38., transformed.0[0][1]);
        approx_eq!(f64, -2., transformed.0[1][0]);
        approx_eq!(f64, -5. - 2. * PI, transformed.0[1][1]);
        approx_eq!(f64, 17., transformed.1[0]);
        approx_eq!(f64, 3. * PI - 19., transformed.1[1]);
    }

    #[test]
    fn test_generator() {
        let mut generator = Generator::new(Euclid::Rotate(PI / 4.).as_affine());

        for i in 0..8 {
            let affine = generator(i);
            let exp_affine = Euclid::Rotate((i as f64) * PI / 4.).as_affine();

            approx_eq!(f64, exp_affine.0[0][0], affine.0[0][0]);
            approx_eq!(f64, exp_affine.0[0][1], affine.0[0][1]);
            approx_eq!(f64, exp_affine.0[1][0], affine.0[1][0]);
            approx_eq!(f64, exp_affine.0[1][1], affine.0[1][1]);
            approx_eq!(f64, exp_affine.1[0], affine.1[0]);
            approx_eq!(f64, exp_affine.1[1], affine.1[1]);
        }
    }

    #[test]
    fn test_affine_fmt() {
        let affine = Affine([[-1., 0.23], [1.1, -2.]], [0.0004, -0.0004]);
        assert_eq!(
            "[-1.000  0.230 0.000]\n[ 1.100 -2.000 0.000]\n[ 0.000  0.000 1.000]",
            format!("{}", affine)
        );
    }
}
