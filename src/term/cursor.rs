pub struct Cursor {
    pub row: usize,
    pub col: usize,
}
impl Cursor {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
    pub fn _reset(&mut self) {
        *self = Self { row: 0, col: 0 };
    }
    pub fn _tuple(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    pub fn _advance_row(&mut self) {
        self.row += 1;
        self.col = 0;
    }
    pub fn _moveit(&mut self, arrow: &str) {
        match arrow {
            //"up" => self.up(),
            //"down" => self.down(),
            "right" => self.right(),
            "left" => self.left(),
            _ => unreachable!(),
        }
    }
    pub fn right(&mut self) {
        self.col += 1;
    }

    pub fn left(&mut self) {
        if self.col != 0 {
            self.col -= 1
        }
    }

    // fn up(&mut self) {
    //     if self.row != 0 {
    //         self.col = 0;
    //         self.row -= 1;
    //     }
    // }

    // fn down(&mut self, stats: &Stats) {
    //     if !self.last_row(stats) {
    //         self.col = 0;
    //         self.row += 1;
    //     }
    // }

    // fn first_spot(&self) -> bool {
    //     (self.row, self.col) == (0, 0)
    // }
    // fn last_spot(&self, stats: &Stats) -> bool {
    //     self.last_row(stats) && self.last_col(stats)
    // }
    // fn last_row(&self, stats: &Stats) -> bool {
    //     self.row == stats.rows_num() - 1
    // }
    // fn last_col(&self, stats: &Stats) -> bool {
    //     self.col == stats.get_row_len(self.row)
    // }
}
