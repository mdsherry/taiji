use std::collections::HashSet;

use crate::{panel::Panel, grid::Grid2};

use super::{Gridlike, ROTATIONS, Rotation};


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Neighbourhood {
    pub offset_x: usize,
    pub offset_y: usize,

    pub contents: Vec<(i8, i8, Panel)>,
}

#[test]
fn neighbour_test() {
    let g = Grid2::new(3, 3);
    let mut n = Neighbourhood::new_around(1, 1, &g);
    let e1 = Neighbourhood {
        offset_x: 1, offset_y: 1, contents: vec![(0, -1, Panel::default()), (-1, 0, Panel::default()), (0, 0, Panel::default()), (1, 0, Panel::default()), (0, 1, Panel::default())]
    };
    assert!(n.same_shape(&e1));
    n.constrain_to_before(1, 2);
    
    assert!(n.same_shape(&e1));
    n.constrain_to_before(2, 1);
    
    let e2 = Neighbourhood {
        offset_x: 1, offset_y: 1, contents: vec![(0, -1, Panel::default()), (-1, 0, Panel::default()), (0, 0, Panel::default()), (1, 0, Panel::default())]
    };
    
    assert!(n.same_shape(&e2));
    n.constrain_to_before(1, 1);
    let e3 = Neighbourhood {
        offset_x: 1, offset_y: 1, contents: vec![(0, -1, Panel::default()), (-1, 0, Panel::default()), (0, 0, Panel::default())]
    };
    assert!(n.same_shape(&e3));
    n.constrain_to_before(0, 1);
    let e4 = Neighbourhood {
        offset_x: 1, offset_y: 1, contents: vec![(0, -1, Panel::default()), (-1, 0, Panel::default())]
    };
    assert!(n.same_shape(&e4));
    let e5 = Neighbourhood {
        offset_x: 1, offset_y: 1, contents: vec![(0, -1, Panel::default())]
    };
    n.constrain_to_before(1, 0);
    assert!(n.same_shape(&e5));
}

#[test]
fn neighbour_test2() {
    let g = Grid2::new(3, 3);
    let n = Neighbourhood::new_around(0, 0, &g);
    assert_eq!(3, n.contents.len());
    let n = Neighbourhood::new_around(2, 2, &g);
    assert_eq!(3, n.contents.len());
    let n = Neighbourhood::new_around(1, 0, &g);
    assert_eq!(4, n.contents.len());
}
impl Neighbourhood {
    pub fn new_around<'a, G: Gridlike<'a>>(x: usize, y: usize, grid: &'a G) -> Self {
        let mut contents = vec![];
        if y > 0 {
            contents.push((0, -1, grid.panel_at(x, y - 1)));
        }
        if x > 0 {
            contents.push((-1, 0, grid.panel_at(x - 1, y)));
        }
        contents.push((0, 0, grid.panel_at(x, y)));
        if x + 1 < grid.width() {
            contents.push((1, 0, grid.panel_at(x + 1, y)));
        }
        if y + 1 < grid.height() {
            contents.push((0, 1, grid.panel_at(x, y + 1)));
        }

        Neighbourhood {
            offset_x: x,
            offset_y: y,
            contents,
        }
    }
    pub fn translate_to(&mut self, x: usize, y: usize) {
        self.offset_x = x;
        self.offset_y = y;
    }
    pub fn inbounds(&self, width: usize, height: usize) -> bool {
        self.contents.iter().all(|(x, y, _)| {
            let x = *x + self.offset_x as i8;
            let y = *y + self.offset_y as i8;
            x >= 0 && x < width as i8 && y >= 0 && y < height as i8
        })
    }
    pub fn inbounds_rotated(&self, width: usize, height: usize) -> bool {
        ROTATIONS.iter().any(|rot| 
            self.contents.iter().all(|(x, y, _)| {
                let (x, y) = rot.rotate((*x, *y));
                let x = x + self.offset_x as i8;
                let y = y + self.offset_y as i8;
                x >= 0 && x < width as i8 && y >= 0 && y < height as i8
            })
        )
    }
    pub fn constrain_to_before(&mut self, upto_x: usize, upto_y: usize) {
        self.contents.retain(|(x, y, p)| {
            let point = ((*y + self.offset_y as i8) as usize, (*x + self.offset_x as i8) as usize);
            p.fixed || point <= (upto_y, upto_x)
        })
    }

    pub fn same_shape(&self, other: &Neighbourhood) -> bool {
        let mut mine: Vec<_> = self.contents.iter().map(|(x, y, _)| (*x, *y)).collect();
        mine.sort_unstable();
        let mut theirs: Vec<_> = other.contents.iter().map(|(x, y, _)| (*x, *y)).collect();
        theirs.sort_unstable();
        mine == theirs
    }
    pub fn same_shape_rotated(&self, other: &Neighbourhood) -> bool {
        let mut mine: Vec<_> = self.contents.iter().map(|(x, y, _)| (*x, *y)).collect();
        mine.sort_unstable();
        for rot in ROTATIONS {
            let mut theirs: Vec<_> = other.contents.iter().map(|(x, y, _)| rot.rotate((*x, *y))).collect();
            theirs.sort_unstable();
            if mine == theirs {
                return true;
            }
        }
        false
    }
    pub fn overlap(&self, other: &Neighbourhood) -> usize {
        let theirs: HashSet<_> = other.contents.iter().map(|(x, y, _)| (*x, *y)).collect();
        let mut count = 0;
        for &(x, y, _) in &self.contents {
            if theirs.contains(&(x, y)) {
                count += 1;
            }
        }
        count
    }
    pub fn rotated_overlap(&self, other: &Neighbourhood, rot: Rotation) -> usize {
        let theirs: HashSet<_> = other.contents.iter().map(|(x, y, _)| (*x, *y)).collect();
        let mut count = 0;
        for &(x, y, _) in &self.contents {
            let target = rot.rotate((x, y));
            if theirs.contains(&target) {
                count += 1;
            }
        }
        count
    }
}
