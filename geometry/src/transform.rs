use crate::affine::{Affine, IDENTITY_AFFINE};
use std::iter;

pub trait Transform {
    fn as_affine(&self) -> Affine;
}

pub trait Transformable<'a> {
    fn transform<T: Transform>(&self, transform: &'a T) -> Self;
}

// reduce_transforms compresses a sequence of transforms into a single affine
// such that the output transform is equivalent to applying the transforms from left to right
// Ex: x.transform(&reduce_transforms(vec![A, B, C])) =~ x.transform(&A).transform(&B).transform(&C), or in matrix notation, =~ C * B * A * x
pub fn reduce_transforms<T: Transform>(transforms: &Vec<T>) -> Affine {
    match iter::once(IDENTITY_AFFINE)
        .chain(transforms.into_iter().map(|t| t.as_affine()))
        .reduce(|a, e| a.transform(&e))
    {
        Some(affine) => affine,
        None => panic!("unable to reduce transforms"),
    }
}
