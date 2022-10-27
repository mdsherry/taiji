use crate::panel::Panel;
use std::{
    collections::HashSet,
    fmt::{Display, Write},
    str::FromStr,
};
use thiserror::Error;

use super::{Gridlike, PanelLoc, PanelLocParseError, Rotation, ROTATIONS};

#[cfg(test)]
mod test;

/// A Neighbourhood represents the largest connected collection of panels with the same 'lit' status, starting from a given point.
/// Since the positions of the panels are stored relative to this point, it's referred to the offset point: the actual grid positions
/// are equal to the position in the neighbourhood plus the offset point.
///
/// Although the [`new_around()`](Neighbourhood::new_around) method exists, the most common way of generating neighbourhoods is through
/// the [`Gridlike::neighbourhood()`](Gridlike::neighbourhood) method.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Neighbourhood {
    pub offset_x: usize,
    pub offset_y: usize,

    pub contents: Vec<(i8, i8, Panel)>,
}

impl Neighbourhood {
    /// Generates a new neighbourhood centered at the given point, and containing all in-bounds
    /// points directly above, below, to the left and to the right of the point.
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
    /// Translates the neighbourhood to a new location, which may result in points being out-of-bounds.
    pub fn translate_to(&mut self, x: usize, y: usize) {
        self.offset_x = x;
        self.offset_y = y;
    }
    #[allow(dead_code)]
    pub fn translated_to(&self, x: usize, y: usize) -> Self {
        let mut rv = self.clone();
        rv.translate_to(x, y);
        rv
    }
    /// Returns an iterator over points in grid-space, rather than offset-space.
    pub fn grid_iter<'a>(&'a self) -> NeighbourhoodGridIter<'a> {
        NeighbourhoodGridIter {
            neighbourhood: self,
            idx: 0,
        }
    }
    
    /// Does the neighbourhood contain a given point (in grid coordinates)?
    pub fn contains(&self, x: usize, y: usize) -> bool {
        let x = x as i8 - self.offset_x as i8;
        let y = y as i8 - self.offset_y as i8;
        self.contents.iter().any(|(xx, yy, _)| *xx == x && *yy == y)
    }

    /// Returns whether all points in the neighbourhood are in-bounds on a grid with the given width and height
    pub fn inbounds(&self, width: usize, height: usize) -> bool {
        self.contents.iter().all(|(x, y, _)| {
            let x = x + self.offset_x as i8;
            let y = y + self.offset_y as i8;
            x >= 0 && x < width as i8 && y >= 0 && y < height as i8
        })
    }
    /// Returns whether there exists a rotation such that all points in the neighbourhood are in-bounds on a grid with the given width and height
    pub fn inbounds_rotated(&self, width: usize, height: usize) -> bool {
        ROTATIONS.iter().any(|rot| {
            self.contents.iter().all(|(x, y, _)| {
                let (x, y) = rot.rotate((*x, *y));
                let x = x + self.offset_x as i8;
                let y = y + self.offset_y as i8;
                x >= 0 && x < width as i8 && y >= 0 && y < height as i8
            })
        })
    }
    /// Constraints the neighbourhood to only points that fall before the given coordinates on a row-by-row scan of the grid.
    pub fn constrain_to_before(&mut self, upto_x: usize, upto_y: usize) {
        self.contents.retain(|(x, y, p)| {
            let point = (
                (*y + self.offset_y as i8) as usize,
                (*x + self.offset_x as i8) as usize,
            );
            p.fixed || point <= (upto_y, upto_x)
        })
    }

    /// Returns whether two neighbourhoods describe the same shape.
    ///
    /// Two shapes must have their offset points in the same location to be considered the same, e.g.
    /// ```text
    /// OXX and XXO
    /// ```
    /// are not the same shape
    pub fn same_shape(&self, other: &Neighbourhood) -> bool {
        let mut mine: Vec<_> = self.contents.iter().map(|(x, y, _)| (*x, *y)).collect();
        mine.sort_unstable();
        let mut theirs: Vec<_> = other.contents.iter().map(|(x, y, _)| (*x, *y)).collect();
        theirs.sort_unstable();
        mine == theirs
    }

    /// Returns whether two neighbourhoods describe the same shape, even at different offsets and orientations
    ///
    /// To count as the same shape, the offset point must be at the same point in both shapes: e.g.
    /// ```text
    /// O   and   X
    /// XX        XO
    /// ```
    /// are not considered to be the same (rotated) shape as their offsets are in different locations.
    pub fn same_shape_rotated(&self, other: &Neighbourhood) -> bool {
        let mut mine: Vec<_> = self.contents.iter().map(|(x, y, _)| (*x, *y)).collect();
        mine.sort_unstable();
        for rot in ROTATIONS {
            let mut theirs: Vec<_> = other
                .contents
                .iter()
                .map(|(x, y, _)| rot.rotate((*x, *y)))
                .collect();
            theirs.sort_unstable();
            if mine == theirs {
                return true;
            }
        }
        false
    }

    /// Returns the overlap between two neighbourhoods, assuming one was translated using
    /// [`translate_to`](Neighbourhood::translate_to) to share the same offset point, and then rotated as indicated
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
    /// Returns the bounds of the neighbourhood relative to the offset point in the order (x_min, y_min, x_max, y_max)
    pub fn bounds(&self) -> (i8, i8, i8, i8) {
        let mut x_min = 0;
        let mut x_max = 0;
        let mut y_min = 0;
        let mut y_max = 0;

        for (x, y, _) in &self.contents {
            x_min = x_min.min(*x);
            x_max = x_max.max(*x);
            y_min = y_min.min(*y);
            y_max = y_max.max(*y);
        }

        (x_min, y_min, x_max, y_max)
    }
}

impl Display for Neighbourhood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            PanelLoc {
                x: self.offset_x,
                y: self.offset_y
            }
        )?;
        let (x_min, y_min, x_max, y_max) = self.bounds();
        for y in y_min..=y_max {
            for x in x_min..=x_max {
                if (x, y) == (0, 0) {
                    f.write_char('O')?;
                } else if let Some((_, _, p)) = self
                    .contents
                    .iter()
                    .find(|(nx, ny, _)| *nx == x && *ny == y)
                {
                    f.write_char(if p.filled { '#' } else { '.' })?;
                } else {
                    f.write_char(' ')?;
                }
            }
            f.write_char('\n')?;
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum NeighbourhoodParseError {
    #[error("Premature end to input")]
    PrematureEndToInput,
    #[error("Error parsing offset: {0}")]
    PanelLocParseError(#[from] PanelLocParseError),
}

impl FromStr for Neighbourhood {
    type Err = NeighbourhoodParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (loc, rest) = s
            .split_once('\n')
            .ok_or(NeighbourhoodParseError::PrematureEndToInput)?;
        let loc: PanelLoc = loc.parse()?;
        let mut contents = vec![];
        let mut origin = (0, 0);
        for (y, line) in rest.lines().enumerate() {
            for (x, cell) in line.chars().enumerate() {
                match cell {
                    'O' => {
                        origin = (x, y);
                        contents.push((x, y));
                    }
                    '#' | '.' => {
                        contents.push((x, y));
                    }
                    _ => (),
                }
            }
        }
        contents.sort_unstable();
        let contents: Vec<_> = contents
            .into_iter()
            .map(|(x, y)| {
                (
                    x as i8 - origin.0 as i8,
                    y as i8 - origin.1 as i8,
                    Panel {
                        filled: true,
                        ..Panel::default()
                    },
                )
            })
            .collect();

        Ok(Neighbourhood {
            offset_x: loc.x,
            offset_y: loc.y,
            contents,
        })
    }
}

pub struct NeighbourhoodGridIter<'a> {
    neighbourhood: &'a Neighbourhood,
    idx: usize,
}
impl<'a> Iterator for NeighbourhoodGridIter<'a> {
    type Item = (usize, usize, Panel);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx > self.neighbourhood.contents.len() {
            None
        } else {
            let (x, y, panel) = self.neighbourhood.contents[self.idx];

            let x = self.neighbourhood.offset_x as i8 + x;
            let y = self.neighbourhood.offset_y as i8 + y;
            assert!(x >= 0);
            assert!(y >= 0);
            self.idx += 1;
            Some((x as usize, y as usize, panel))
        }
    }
}
