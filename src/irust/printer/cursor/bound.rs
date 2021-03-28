use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Bound {
    pub bound: HashMap<usize, BoundType>,
    pub width: usize,
    pub height: usize,
    sorted: Vec<(usize, BoundType)>,
}
#[derive(Debug, Clone, Copy)]
pub enum BoundType {
    Bounded(usize),
    Unbounded,
}
impl Default for BoundType {
    fn default() -> Self {
        Self::Unbounded
    }
}

// impl From<BoundType> for usize {
// fn from(bound: BoundType) -> Self {
// match bound {
// BoundType::Bounded(b) => b,
// BoundType::Unbounded => width,
// }
// }
// }

impl Bound {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            bound: HashMap::new(),
            width,
            height,
            sorted: vec![],
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.width, self.height);
    }

    pub fn get_bound(&self, row: usize) -> &usize {
        self.bound_to_usize(self.bound.get(&row).unwrap_or(&BoundType::Unbounded))
        //self.bound.get(row).unwrap()
    }
    fn bound_to_usize<'a>(&'a self, bound: &'a BoundType) -> &usize {
        match bound {
            BoundType::Bounded(b) => b,
            BoundType::Unbounded => &self.width,
        }
    }
    // pub fn _get_mut_bound(&mut self, row: usize) -> &mut usize {
    //     self.bound.get_mut(row).unwrap()
    // }

    pub fn set_bound(&mut self, row: usize, col: usize) {
        *self.bound.entry(row).or_insert(BoundType::Unbounded) = BoundType::Bounded(col);
    }

    // pub fn _insert_bound(&mut self, row: usize, col: usize) {
    //     // circular buffer
    //     self.bound.insert(row, col);
    //     self.bound[0] = self.bound.pop().unwrap();
    // }

    /// sum bounds from start inclusive to end exclusive
    /// Adds +1 to bounds because of 0/1 indexing mismatch ?
    pub fn bounds_sum(&mut self, start_row: usize, end_row: usize) -> usize {
        self.sorted = self.bound.iter().map(|(i, v)| (*i, *v)).collect();
        self.sorted.sort_by_key(|(idx, _)| *idx);
        dbg!(&self.sorted);
        self.sorted
            .iter()
            .map(|(_, v)| v)
            .take(end_row - start_row)
            .map(|b| dbg!(self.bound_to_usize(b) + 1)) //- super::INPUT_START_COL)
            .sum()
    }
}

#[test]
fn test_bound() {
    let w = 50;
    let h = 100;

    let mut b = Bound::new(w, h);
    b.set_bound(50, 10);
    b.set_bound(51, 11);
    b.set_bound(52, 12);
    assert_eq!(b.get_bound(50), &10);
    assert_eq!(b.get_bound(51), &11);
    assert_eq!(b.get_bound(52), &12);
    assert_eq!(b.bounds_sum(50, 52), 10 + 11 + 1 * 2);
    //line 53 is not set yet so it shouldn't impact the math
    assert_eq!(b.bounds_sum(50, 53), 10 + 11 + 12 + 1 * 3);
    //line 53 is unbounded (= width)
    assert_eq!(*b.get_bound(53), w);
}
