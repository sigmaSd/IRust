#[derive(Debug, Default, Clone)]
pub struct Bound {
    pub bound: Vec<usize>,
    pub width: usize,
    pub height: usize,
}

impl Bound {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            bound: vec![width - 1; height],
            width,
            height,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.width, self.height);
    }

    pub fn get_bound(&self, row: usize) -> usize {
        self.bound.get(row).copied().unwrap_or(self.width - 1)
    }

    pub fn _get_mut_bound(&mut self, row: usize) -> &mut usize {
        self.bound.get_mut(row).unwrap()
    }

    pub fn set_bound(&mut self, row: usize, col: usize) {
        self.bound[row] = col;
    }

    pub fn _insert_bound(&mut self, row: usize, col: usize) {
        // circular buffer
        self.bound.insert(row, col);
        self.bound[0] = self.bound.pop().unwrap();
    }

    pub fn bounds_sum(&self, start_row: usize, end_row: usize, prompt_len: usize) -> usize {
        self.bound
            .iter()
            .take(end_row)
            .skip(start_row)
            .map(|b| b + 1 - prompt_len)
            .sum()
    }
}
