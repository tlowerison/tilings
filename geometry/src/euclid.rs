use crate::{
    affine::{Affine, IDENTITY_AFFINE},
    transform::{Transform, Transformable},
};

pub enum Euclid {
    Composite(Affine),
    Translate((f64, f64)), // parameterizes (dx,dy) to move an object by (i.e. underlying reference frame is not shifted)
    Rotate(f64), // parameterizes angle through the origin which an object will be rotated by - expects radians
    Flip(f64),   // parameterizes angle through the origin of flip line - expects radians
}

impl Transform for Euclid {
    fn as_affine(&self) -> Affine {
        match self {
            Euclid::Composite(affine) => affine.clone(),
            Euclid::Translate((dx, dy)) => Affine(IDENTITY_AFFINE.0, [*dx, *dy]),
            Euclid::Rotate(radians) => {
                let radians = *radians;
                let cos = radians.cos();
                let sin = radians.sin();
                Affine([[cos, -sin], [sin, cos]], IDENTITY_AFFINE.1)
            }
            Euclid::Flip(radians) => {
                let radians = *radians;
                let cos = radians.cos();
                let sin = radians.sin();
                Affine([[cos, sin], [sin, -cos]], IDENTITY_AFFINE.1)
            }
        }
    }
}

impl<'a> Transformable<'a> for Euclid {
    fn transform<T: Transform>(&self, transform: &'a T) -> Self {
        let affine = &transform.as_affine();
        if let Euclid::Composite(t) = self {
            return Euclid::Composite(t.transform(affine));
        } else {
            return Euclid::Composite(self.as_affine().transform(affine));
        }
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
