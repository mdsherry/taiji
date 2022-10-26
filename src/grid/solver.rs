use super::{Grid, Gridlike};

impl Grid {
    pub fn solve_from(&mut self, mut x: usize, mut y: usize) -> Option<Grid> {
        if self.is_solved() {
            return Some(self.clone());
        }
        if x > 0 {
            if !self.solveable(x - 1, y) {
                return None;
            }
        }
        if x >= self.width {
            x = 0;
            y += 1;
        }
        if y >= self.height {
            return None;
        }

        if self.fixed_at(x, y) {
            if self.is_solved() {
                return Some(self.clone());
            } else {
                self.solve_from(x + 1, y)
            }
        } else {
            let state = self.lit;
            self.set_lit_at(x, y, false);
            let s1 = self.solve_from(x + 1, y);
            if s1.is_some() {
                return s1;
            }
            self.set_lit_at(x, y, true);
            let s1 = self.solve_from(x + 1, y);
            if s1.is_some() {
                return s1;
            }
            self.lit = state;
            None
        }
    }
    fn solveable(&self, upto_x: usize, upto_y: usize) -> bool {
        for (x, y, panel) in self.iter() {
            if !panel.satisfiable(x, y, upto_x, upto_y, self) {
                return false;
            }
        }
        true
    }
}
