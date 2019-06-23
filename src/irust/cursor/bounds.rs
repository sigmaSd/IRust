use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Bounds {
    bd: HashMap<usize, (usize, usize)>,
}

impl Bounds {
    pub fn new(y: usize, (lb, hb): (usize, usize)) -> Self {
        let mut bd = HashMap::new();
        bd.insert(y, (lb, hb));

        Self { bd }
    }

    pub fn lower_bound(&self, y: usize) -> usize {
        self.bd[&y].0
    }

    pub fn upper_bound(&self, y: usize) -> usize {
        self.bd[&y].1
    }

    pub fn get_mut(&mut self, y: usize) -> Option<&mut (usize, usize)> {
        self.bd.get_mut(&y)
    }

    pub fn insert(&mut self, y: usize, (lb, hb): (usize, usize)) {
        self.bd.insert(y, (lb, hb));
    }

    pub fn contains(&mut self, y: usize) -> bool {
        self.bd.contains_key(&y)
    }

    pub fn shift_keys_left(&mut self, n: usize) {
        let mut new_h = std::collections::HashMap::new();

        for i in 0..n {
            self.bd.remove_entry(&i);
        }

        for (k, v) in &self.bd {
            new_h.insert(k - 1, *v);
        }

        self.bd = new_h;
    }
}
