use float_cmp::ApproxEq;
use itertools;
use num_traits::{cast::NumCast, Float};
use std::{
    collections::hash_map::DefaultHasher,
    f64::consts::TAU,
    hash::{Hash, Hasher},
};

// DEFAULT_F64_MARGIN guarantees that a pair of points with neither coordinate
// differing by more than F64_MARGIN.0 / 10. will hash to the same value.
pub const DEFAULT_F64_MARGIN: (f64, i64) = (0.000_001, 5);

// approx_eq rewrites float_cmp's macro approx_eq! with an additional assertion in case
// the two values are not approximately equal because otherwise we lose the info of
// what the two compared values were in the test output.
#[macro_export]
macro_rules! approx_eq {
    ($typ:ty, $lhs:expr, $rhs:expr) => {
        {
            if !<$typ as float_cmp::ApproxEq>::approx_eq($lhs, $rhs, (0.000_001, 5)) {
                assert_eq!($lhs, $rhs)
            }
        }
    };
    ($typ:ty, $lhs:expr, $rhs:expr $(, $set:ident = $val:expr)*) => {
        {
            let m = <$typ as float_cmp::ApproxEq>::Margin::zero()$(.$set($val))*;
            if !<$typ as float_cmp::ApproxEq>::approx_eq($lhs, $rhs, m) {
                assert_eq!($lhs, $rhs)
            }
        }
    };
    ($typ:ty, $lhs:expr, $rhs:expr, $marg:expr) => {
        {
            if !<$typ as float_cmp::ApproxEq>::approx_eq($lhs, $rhs, $marg) {
                assert_eq!($lhs, $rhs)
            }
        }
    };
}

// calc_hash computes and outputs a values hash
pub fn calc_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

// fmt_float truncates digits from a float
pub fn fmt_float<F: Float>(f: F, decimal_precision: u32) -> String {
    let pow = 10_i32.pow(decimal_precision);
    let i = (f * NumCast::from(pow).unwrap()).round().to_i32().unwrap();
    format!(
        "{}{}.{}",
        if i < 0 { "-" } else { "" },
        (i / pow).abs(),
        if decimal_precision == 0 {
            String::from("")
        } else {
            format!(
                "{:0<precision$}",
                (i % pow).abs(),
                precision = (decimal_precision as usize)
            )
        }
    )
}

// hash_f64 multiplies f by a power of 10 then cuts off all fractional digits by rounding
pub fn hash_float<F: Float>(f: F, decimal_precision: u32) -> i32 {
    (f * NumCast::from(10_i32.pow(decimal_precision)).unwrap())
        .round()
        .to_i32()
        .unwrap()
}

// rad converts any radian value into the equivalent radian value in the range [0, τ]
pub fn rad(f: f64) -> f64 {
    let rad = (TAU + (f % TAU)) % TAU;
    if rad.approx_eq(TAU, DEFAULT_F64_MARGIN) { 0. } else { rad }
}

// rev_iter returns a conditionally reversed iterator
pub fn rev_iter<I>(
    iter: impl DoubleEndedIterator<Item = I>,
    should_reverse: bool,
) -> impl Iterator<Item = I> {
    if !should_reverse {
        itertools::Either::Left(iter)
    } else {
        itertools::Either::Right(iter.rev())
    }
}

// to_rad converts degrees to radians
pub fn to_rad(f: f64) -> f64 {
    rad(f / 360. * TAU)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::NumCast;
    use std::f64::consts::PI;

    #[test]
    fn test_fmt_float() {
        let float = NumCast::from(PI).unwrap();

        assert_eq!("3.", fmt_float::<f32>(float, 0));
        assert_eq!("3.1", fmt_float::<f32>(float, 1));
        assert_eq!("3.14", fmt_float::<f32>(float, 2));
        assert_eq!("3.142", fmt_float::<f32>(float, 3));
        assert_eq!("3.1416", fmt_float::<f32>(float, 4));

        let float = NumCast::from(PI).unwrap();

        assert_eq!("3.", fmt_float::<f64>(float, 0));
        assert_eq!("3.1", fmt_float::<f64>(float, 1));
        assert_eq!("3.14", fmt_float::<f64>(float, 2));
        assert_eq!("3.142", fmt_float::<f64>(float, 3));
        assert_eq!("3.1416", fmt_float::<f64>(float, 4));

        let float = NumCast::from(1.).unwrap();

        assert_eq!("1.", fmt_float::<f32>(float, 0));
        assert_eq!("1.0", fmt_float::<f32>(float, 1));
        assert_eq!("1.00", fmt_float::<f32>(float, 2));
        assert_eq!("1.000", fmt_float::<f32>(float, 3));
        assert_eq!("1.0000", fmt_float::<f32>(float, 4));

        let float = NumCast::from(1.).unwrap();

        assert_eq!("1.", fmt_float::<f64>(float, 0));
        assert_eq!("1.0", fmt_float::<f64>(float, 1));
        assert_eq!("1.00", fmt_float::<f64>(float, 2));
        assert_eq!("1.000", fmt_float::<f64>(float, 3));
        assert_eq!("1.0000", fmt_float::<f64>(float, 4));
    }

    #[test]
    fn test_hash_float() {
        let float = NumCast::from(PI).unwrap();

        assert_eq!(3, hash_float::<f32>(float, 0));
        assert_eq!(31, hash_float::<f32>(float, 1));
        assert_eq!(314, hash_float::<f32>(float, 2));
        assert_eq!(3142, hash_float::<f32>(float, 3));
        assert_eq!(31416, hash_float::<f32>(float, 4));

        assert_eq!(-3, hash_float::<f32>(-float, 0));
        assert_eq!(-31, hash_float::<f32>(-float, 1));
        assert_eq!(-314, hash_float::<f32>(-float, 2));
        assert_eq!(-3142, hash_float::<f32>(-float, 3));
        assert_eq!(-31416, hash_float::<f32>(-float, 4));

        let float = NumCast::from(PI).unwrap();

        assert_eq!(3, hash_float::<f64>(float, 0));
        assert_eq!(31, hash_float::<f64>(float, 1));
        assert_eq!(314, hash_float::<f64>(float, 2));
        assert_eq!(3142, hash_float::<f64>(float, 3));
        assert_eq!(31416, hash_float::<f64>(float, 4));

        assert_eq!(-3, hash_float::<f64>(-float, 0));
        assert_eq!(-31, hash_float::<f64>(-float, 1));
        assert_eq!(-314, hash_float::<f64>(-float, 2));
        assert_eq!(-3142, hash_float::<f64>(-float, 3));
        assert_eq!(-31416, hash_float::<f64>(-float, 4));
    }

    #[test]
    fn test_rev_iter() {
        let mut iter = rev_iter(0..10, false);
        assert_eq!(0_usize, iter.next().unwrap());
        let mut iter = rev_iter(0..10, true);
        assert_eq!(9_usize, iter.next().unwrap());
    }
}
