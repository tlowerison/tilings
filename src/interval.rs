use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

pub const LEFT_IDENTIY_INTERVAL: Interval = Interval(0,0,0);
pub const RIGHT_IDENTIY_INTERVAL: Interval = Interval(0,0,0);

pub struct Interval(u8, u8, u64);

// SizedInterval is only concerned with size in terms of ordering.
pub struct SizedInterval(pub(crate) Interval);

// PositionedInterval is only concerned with positioning in terms of ordering.
pub struct PositionedInterval(pub(crate) Interval);


impl Interval {
    pub fn new(first: u8, last: u8) -> Interval {
        assert!(first < last);
        let size = last - first;
        Interval(first, last, 2u64.pow(size as u32) - 1 << (64 - first - size))
    }
    pub fn first(&self) -> u8 { self.0 }
    pub fn last(&self) -> u8 { self.1 }
    pub fn size(&self) -> u8 { self.1 - self.0 }
    pub fn set_first(&mut self, first: u8) { self.0 = first; }
    pub fn set_last(&mut self, last: u8) { self.1 = last; }
}

impl Clone for Interval {
    fn clone(&self) -> Self { Interval(self.0, self.1, self.2) }
}


impl SizedInterval {
    pub fn new(first: u8, last: u8) -> SizedInterval { SizedInterval(Interval::new(first, last)) }
    pub fn first(&self) -> u8 { self.0.0 }
    pub fn last(&self) -> u8 { self.0.1 }
    pub fn size(&self) -> u8 { self.0.size() }
}

impl Clone for SizedInterval {
    fn clone(&self) -> Self { SizedInterval(self.0.clone()) }
}

impl Eq for SizedInterval {}

impl PartialEq for SizedInterval {
    fn eq(&self, other: &Self) -> bool {
        self.0.size() == other.0.size()
    }
}

impl Ord for SizedInterval {
    fn cmp(&self, other: &Self) -> Ordering {
        let ordering = self.0.size().cmp(&other.0.size());
        if let Ordering::Equal = ordering {
            return self.0.0.cmp(&other.0.0)
        }
        ordering
    }
}

impl PartialOrd for SizedInterval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}


impl PositionedInterval {
    pub fn new(first: u8, last: u8) -> PositionedInterval { PositionedInterval(Interval::new(first, last)) }
    pub fn first(&self) -> u8 { self.0.0 }
    pub fn last(&self) -> u8 { self.0.1 }
    pub fn size(&self) -> u8 { self.0.size() }
}

impl Clone for PositionedInterval {
    fn clone(&self) -> Self { PositionedInterval(self.0.clone()) }
}

impl Eq for PositionedInterval {}

impl PartialEq for PositionedInterval {
    fn eq(&self, other: &Self) -> bool {
        self.0.0 == other.0.0 && self.0.1 == other.0.1
    }
}

impl Ord for PositionedInterval {
    fn cmp(&self, other: &Self) -> Ordering {
        let ordering = self.0.0.cmp(&other.0.0);
        if let Ordering::Equal = ordering {
            return self.0.1.cmp(&other.0.1)
        }
        ordering
    }
}

impl PartialOrd for PositionedInterval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
