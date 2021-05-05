use float_cmp::*;
use crate::common::*;

pub struct Segment(pub(crate) Point, pub(crate) Point);

impl Segment {
    // overlaps calculates whether two segments are linearly dependent and intersecting
    pub fn overlaps(&self, other: &Segment) -> bool {
        let dself = &self.1 - &self.0;
        let dother = &other.1 - &other.0;
        if dother.1.atan2(dother.0).approx_eq(dself.1.atan2(dself.0), F64Margin::default()) {
            return true
        }

        let mut rotated_self = [
            self.0.transform(&Euclid::Rotate(-self.0.1.atan2(self.0.0))).0,
            self.1.transform(&Euclid::Rotate(-self.1.1.atan2(self.1.0))).0,
        ];
        rotated_self.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut rotated_other = [
            self.0.transform(&Euclid::Rotate(-other.0.1.atan2(other.0.0))).0,
            self.1.transform(&Euclid::Rotate(-other.1.1.atan2(other.1.0))).0,
        ];
        rotated_other.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut intervals = [rotated_self, rotated_other];
        intervals.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());

        match intervals[0][1].partial_cmp(&intervals[1][0]) {
            Some(ordering) => ordering == core::cmp::Ordering::Greater,
            None => panic!("couldn't compare {} againts {}", intervals[1][0], intervals[0][1]),
        }
    }
}

pub struct Segments(pub(crate) Vec<Segment>);
