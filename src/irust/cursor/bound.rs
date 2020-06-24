pub struct Bound {
    pub bound: Vec<usize>,
    hidden_bounds: Vec<usize>,
    pub width: usize,
    pub height: usize,
}

impl Bound {
    pub fn new(width: usize, height: usize) -> Self {
        let mut bound = Vec::new();
        for _ in 0..height {
            bound.push(width - 1);
        }

        Self {
            bound,
            hidden_bounds: vec![],
            width,
            height,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.width, self.height);
    }

    pub fn get_bound(&self, row: usize) -> &usize {
        self.bound.get(row).unwrap()
    }
    pub fn _get_mut_bound(&mut self, row: usize) -> &mut usize {
        self.bound.get_mut(row).unwrap()
    }

    pub fn set_bound(&mut self, row: usize, col: usize) {
        if row >= self.bound.len() {
            self.hidden_bounds.push(self.bound.remove(0));
            self.bound.push(col);
        } else {
            self.bound[row] = col;
        }
    }

    pub fn _insert_bound(&mut self, row: usize, col: usize) {
        // circular buffer
        self.bound.insert(row, col);
        self.bound[0] = self.bound.pop().unwrap();
    }

    pub fn bounds_sum(&self, mut start_row: isize, end_row: usize) -> usize {
        // let mut total_bounds: Vec<usize> = vec![];
        // total_bounds.extend(self.hidden_bounds.iter());
        // total_bounds.extend(self.bound.iter());
        //
        // // push everyhtinh to positive range (0..)
        // if start_row < 0 {
        //     end_row = end_row + start_row.abs() as usize;
        //     start_row = 0
        // }

        start_row = std::cmp::max(0, start_row);
        self.bound
            .iter()
            .take(end_row)
            .skip(start_row as usize)
            .map(|b| b + 1 - super::INPUT_START_COL)
            .sum()
    }
}
