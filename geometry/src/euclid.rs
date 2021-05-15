use crate::{
    affine::{Affine, IDENTITY_AFFINE},
    point::DISPLAY_PRECISION,
    transform::{Transform, Transformable},
};
use common::fmt_float;
use std::f64::consts::PI;

pub enum Euclid {
    Translate((f64, f64)), // parameterizes (dx,dy) to move an object by (i.e. underlying reference frame is not shifted)
    Rotate(f64), // parameterizes angle through the origin which an object will be rotated by - expects radians
    Flip(f64),   // parameterizes angle through the origin of flip line - expects radians
    Composite(Affine),
}

impl Transform for Euclid {
    fn as_affine(&self) -> Affine {
        match self {
            Euclid::Translate((dx, dy)) => Affine(IDENTITY_AFFINE.0, [*dx, *dy]),
            Euclid::Rotate(radians) => {
                let radians = *radians;
                let cos = radians.cos();
                let sin = radians.sin();
                Affine([[cos, -sin], [sin, cos]], IDENTITY_AFFINE.1)
            }
            Euclid::Flip(radians) => {
                let radians = 2. * radians;
                let cos = radians.cos();
                let sin = radians.sin();
                Affine([[cos, sin], [sin, -cos]], IDENTITY_AFFINE.1)
            }
            Euclid::Composite(affine) => affine.clone(),
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
            Euclid::Translate((dx, dy)) => write!(
                f,
                "T({}, {})",
                fmt_float(*dx, DISPLAY_PRECISION),
                fmt_float(*dy, DISPLAY_PRECISION)
            ),
            Euclid::Rotate(radians) => {
                write!(f, "R({}π)", fmt_float(radians / PI, DISPLAY_PRECISION))
            }
            Euclid::Flip(radians) => {
                write!(f, "F({}π)", fmt_float(radians / PI, DISPLAY_PRECISION))
            }
            Euclid::Composite(affine) => write!(f, "{}", affine),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::approx_eq;
    use std::f64::consts::PI;

    #[test]
    fn test_euclid_as_affine() {
        let affine = Euclid::Translate((2., 3.)).as_affine();
        approx_eq!(f64, 1., affine.0[0][0]);
        approx_eq!(f64, 0., affine.0[0][1]);
        approx_eq!(f64, 0., affine.0[1][0]);
        approx_eq!(f64, 1., affine.0[1][1]);
        approx_eq!(f64, 2., affine.1[0]);
        approx_eq!(f64, 3., affine.1[1]);

        let rotation = 2. * PI / 3.;
        let affine = Euclid::Rotate(rotation).as_affine();
        approx_eq!(f64, rotation.cos(), affine.0[0][0]);
        approx_eq!(f64, -rotation.sin(), affine.0[0][1]);
        approx_eq!(f64, rotation.sin(), affine.0[1][0]);
        approx_eq!(f64, rotation.cos(), affine.0[1][1]);
        approx_eq!(f64, 0., affine.1[0]);
        approx_eq!(f64, 0., affine.1[1]);

        let x_axis = PI / 3.;
        let affine = Euclid::Flip(x_axis).as_affine();
        approx_eq!(f64, (2. * x_axis).cos(), affine.0[0][0]);
        approx_eq!(f64, (2. * x_axis).sin(), affine.0[0][1]);
        approx_eq!(f64, (2. * x_axis).sin(), affine.0[1][0]);
        approx_eq!(f64, -(2. * x_axis).cos(), affine.0[1][1]);
        approx_eq!(f64, 0., affine.1[0]);
        approx_eq!(f64, 0., affine.1[1]);

        let affine = Euclid::Composite(Affine([[1., 2.], [3., 4.]], [5., 6.])).as_affine();
        approx_eq!(f64, 1., affine.0[0][0]);
        approx_eq!(f64, 2., affine.0[0][1]);
        approx_eq!(f64, 3., affine.0[1][0]);
        approx_eq!(f64, 4., affine.0[1][1]);
        approx_eq!(f64, 5., affine.1[0]);
        approx_eq!(f64, 6., affine.1[1]);
    }

    #[test]
    fn test_euclid_transform() {
        let euclid = Euclid::Translate((1., 1.))
            .transform(&Euclid::Rotate(PI / 4.))
            .transform(&Euclid::Translate((-1., -1.)));
        match euclid {
            Euclid::Translate(_) => assert!(false),
            Euclid::Rotate(_) => assert!(false),
            Euclid::Flip(_) => assert!(false),
            Euclid::Composite(affine) => {
                let s = 1. / 2_f64.sqrt();
                approx_eq!(f64, s, affine.0[0][0]);
                approx_eq!(f64, -s, affine.0[0][1]);
                approx_eq!(f64, s, affine.0[1][0]);
                approx_eq!(f64, s, affine.0[1][1]);
                approx_eq!(f64, -1., affine.1[0]);
                approx_eq!(f64, 2. * s - 1., affine.1[1]);
            }
        }
    }

    #[test]
    fn test_euclid_fmt() {
        assert_eq!(
            "T(1.33, 4.89)",
            format!("{}", Euclid::Translate((1.3333, 4.8888)))
        );
        assert_eq!("R(0.50π)", format!("{}", Euclid::Rotate(PI / 2.)));
        assert_eq!("F(-0.33π)", format!("{}", Euclid::Flip(-PI / 3.)));
        assert_eq!(
            format!("{}", IDENTITY_AFFINE),
            format!("{}", Euclid::Composite(IDENTITY_AFFINE))
        );
    }
}
