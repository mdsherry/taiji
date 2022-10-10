use super::{Grid, Grid2, Gridlike};

impl Grid {
    pub fn solve_from(&mut self, mut x: usize, mut y: usize) -> Option<Grid> {
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
        if !self.contents[x][y].fixed {
            self.contents[x][y].filled = false;
            if self.is_solved() {
                return Some(self.clone())
            }
        }
        self.solve_from(x + 1, y).or_else(|| {
            if !self.contents[x][y].fixed {
                self.contents[x][y].filled = true;
                if self.is_solved() {
                    Some(self.clone())
                } else {
                    self.solve_from(x + 1, y)
                }
            } else {
                None
            }
        })
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


impl Grid2 {
    
    pub fn solve_from(&mut self, mut x: usize, mut y: usize) -> Option<Grid2> {
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
        if !self.fixed_at(x, y) {
            self.set_lit_at(x, y, false);
            if self.is_solved() {
                return Some(self.clone())
            }
        }
        self.solve_from(x + 1, y).or_else(|| {
            if !self.fixed_at(x, y) {
                self.set_lit_at(x, y, true);
                if self.is_solved() {
                    Some(self.clone())
                } else {
                    self.solve_from(x + 1, y)
                }
            } else {
                None
            }
        })
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