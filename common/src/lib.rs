use itertools;
use num_traits::{Float, cast::NumCast};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

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
            format!("{}", (i % pow).abs())
        }
    )
}

// hash_f64 multiplies f by a power of 10 then cuts off all fractional digits by rounding
pub fn hash_float<F: Float>(f: F, decimal_precision: u32) -> i32 {
    (f * NumCast::from(10_i32.pow(decimal_precision)).unwrap()).round().to_i32().unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::NumCast;
    use std::f64::consts::PI;

    #[test]
    fn test_fmt_float() {
        let pi = NumCast::from(PI).unwrap();

        assert_eq!("3.", fmt_float::<f32>(pi, 0));
        assert_eq!("3.1", fmt_float::<f32>(pi, 1));
        assert_eq!("3.14", fmt_float::<f32>(pi, 2));
        assert_eq!("3.142", fmt_float::<f32>(pi, 3));
        assert_eq!("3.1416", fmt_float::<f32>(pi, 4));

        let pi = NumCast::from(PI).unwrap();

        assert_eq!("3.", fmt_float::<f64>(pi, 0));
        assert_eq!("3.1", fmt_float::<f64>(pi, 1));
        assert_eq!("3.14", fmt_float::<f64>(pi, 2));
        assert_eq!("3.142", fmt_float::<f64>(pi, 3));
        assert_eq!("3.1416", fmt_float::<f64>(pi, 4));
    }

    #[test]
    fn test_hash_float() {
        let pi = NumCast::from(PI).unwrap();

        assert_eq!(3, hash_float::<f32>(pi, 0));
        assert_eq!(31, hash_float::<f32>(pi, 1));
        assert_eq!(314, hash_float::<f32>(pi, 2));
        assert_eq!(3142, hash_float::<f32>(pi, 3));
        assert_eq!(31416, hash_float::<f32>(pi, 4));

        assert_eq!(-3, hash_float::<f32>(-pi, 0));
        assert_eq!(-31, hash_float::<f32>(-pi, 1));
        assert_eq!(-314, hash_float::<f32>(-pi, 2));
        assert_eq!(-3142, hash_float::<f32>(-pi, 3));
        assert_eq!(-31416, hash_float::<f32>(-pi, 4));

        let pi = NumCast::from(PI).unwrap();

        assert_eq!(3, hash_float::<f64>(pi, 0));
        assert_eq!(31, hash_float::<f64>(pi, 1));
        assert_eq!(314, hash_float::<f64>(pi, 2));
        assert_eq!(3142, hash_float::<f64>(pi, 3));
        assert_eq!(31416, hash_float::<f64>(pi, 4));

        assert_eq!(-3, hash_float::<f64>(-pi, 0));
        assert_eq!(-31, hash_float::<f64>(-pi, 1));
        assert_eq!(-314, hash_float::<f64>(-pi, 2));
        assert_eq!(-3142, hash_float::<f64>(-pi, 3));
        assert_eq!(-31416, hash_float::<f64>(-pi, 4));
    }

    #[test]
    fn test_rev_iter() {
        let mut iter = rev_iter(0..10, false);
        assert_eq!(0_usize, iter.next().unwrap());
        let mut iter = rev_iter(0..10, true);
        assert_eq!(9_usize, iter.next().unwrap());
    }
}
