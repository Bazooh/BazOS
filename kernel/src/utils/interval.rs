use crate::utils::debug::DebugHex;
use alloc::vec::Vec;

#[derive(Debug)]
pub struct Interval {
    start: DebugHex<u64>,
    end: DebugHex<u64>,
}

impl Interval {
    pub fn with_size(start: u64, size: u64) -> Self {
        Interval {
            start: DebugHex(start),
            end: DebugHex(start + size),
        }
    }

    pub fn overlap(&self, other: &Interval) -> bool {
        self.start.0 <= other.end.0 && self.end.0 >= other.start.0
    }

    pub fn size(&self) -> u64 {
        self.end.0 - self.start.0
    }

    pub fn start(&self) -> u64 {
        self.start.0
    }
}

pub fn merge_intervals(mut intervals: Vec<Interval>) -> Vec<Interval> {
    intervals.sort_by(|a, b| a.start().cmp(&b.start()));

    let mut merged: Vec<Interval> = Vec::new();
    for interval in intervals {
        if let Some(last) = merged.last_mut() {
            if last.overlap(&interval) {
                last.end = interval.end;
            } else {
                merged.push(interval);
            }
        } else {
            merged.push(interval);
        }
    }
    merged
}
