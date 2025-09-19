pub struct SlidingWindow<T> {
    vec: Vec<T>,
    start: usize,
    end: usize,
}

impl<T> SlidingWindow<T> {
    pub fn new() -> Self {
        Self {
            vec: vec![],
            start: 0,
            end: 1,
        }
    }

    pub fn push(&mut self, data: T) {
        self.vec.push(data);
    }

    pub fn build(data_vec: Vec<T>) -> Self {
        Self {
            vec: data_vec,
            start: 0,
            end: 0,
        }
    }

    pub fn as_slice(&mut self) -> Option<&mut [T]> {
        if self.end <= self.start + 1 {
            return None;
        }
        if self.end > self.vec.len() {
            return None;
        }
        Some(&mut self.vec[self.start..self.end])
    }

    pub fn start_move_while<F: Fn(&T) -> bool>(&mut self, f: F) {
        while self.start < self.end - 1 && self.start < self.vec.len() && f(&self.vec[self.start]) {
            self.start += 1;
        }
    }

    pub fn end_move_while<F: Fn(&T) -> bool>(&mut self, f: F) {
        while self.end < self.vec.len() && f(&self.vec[self.end]) {
            self.end += 1;
        }
    }

    pub fn is_end(&self) -> bool {
        self.start >= self.vec.len() - 1
    }
}

impl<T> Default for SlidingWindow<T> {
    fn default() -> Self {
        Self::new()
    }
}

