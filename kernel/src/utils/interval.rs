use alloc::vec::Vec;

#[derive(Debug)]
pub struct Interval {
    start: usize,
    end: usize,
}

impl Interval {
    pub fn new(start: usize, end: usize) -> Option<Self> {
        if start > end {
            return None;
        }
        Some(Interval { start, end })
    }

    pub fn with_size(start: usize, size: usize) -> Self {
        Interval {
            start,
            end: start + size,
        }
    }

    pub fn overlap(&self, other: &Interval) -> bool {
        self.start <= other.end && self.end >= other.start
    }

    pub fn contains(&self, point: usize) -> bool {
        self.start <= point && self.end >= point
    }

    pub fn size(&self) -> usize {
        self.end - self.start
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }
}

pub fn merge_intervals(mut intervals: Vec<Interval>) -> Vec<Interval> {
    intervals.sort_by(|a, b| a.start.cmp(&b.start));

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
