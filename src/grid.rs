use std::{collections::HashSet, num::ParseIntError, str::FromStr, fmt::{Display, Write}};

use thiserror::Error;

use crate::panel::{self, Panel, Symbol};
mod solver;

#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub contents: Vec<Vec<Panel>>,
}

pub struct Neighbourhood {
    pub offset_x: usize,
    pub offset_y: usize,

    pub contents: Vec<(i8, i8, Panel)>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Rotation {
    D0,
    D90,
    D180,
    D270,
}
pub static ROTATIONS: [Rotation; 4] = [Rotation::D0, Rotation::D90, Rotation::D180, Rotation::D270];
impl Rotation {
    pub fn rotate(self, (x, y): (i8, i8)) -> (i8, i8) {
        match self {
            Rotation::D0 => (x, y),
            Rotation::D270 => (y, -x),
            Rotation::D180 => (-x, -y),
            Rotation::D90 => (-y, x),
        }
    }
}

impl Neighbourhood {
    pub fn entirely_before(&self, upto_x: usize, upto_y: usize) -> bool {
        if (self.offset_y, self.offset_x) > (upto_y, upto_x) {
            return false;
        }
        !self.contents.iter().any(|(x, y, _)| {
            let point = (*y as usize + self.offset_y, *x as usize + self.offset_x);
            point > (upto_y, upto_x)
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

impl Display for Panel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.filled {
            f.write_char('X')?;
        } else {
            f.write_char('.')?;
        }
        if self.fixed {
            f.write_char('!')?;
        }
        match self.symbol {
            Symbol::Plain => (),
            Symbol::Pips { count, color } => {
                write!(f, "C{}{count}", color.code())?;
            }
            Symbol::Line { diagonal, color } => {
                f.write_char(if diagonal { '/' } else { '-' })?;
                f.write_char(color.code())?;
            }
            Symbol::Lozange { color } => {
                f.write_char('O')?;
                f.write_char(color.code())?;
            }
            Symbol::Petals { count } => write!(f, "F{count}")?,
        }
        Ok(())
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        write!(f, "{} {}\n", self.width, self.height)?;

        for y in 0..self.height {
            for row in &self.contents {
                let panel = row[y];
                panel.fmt(f)?;
                f.write_char(' ')?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

#[derive(Debug, Error, Clone)]
pub enum GridParseError {
    #[error("Empty input")]
    EmptyInput,
    #[error("Missing width")]
    MissingWidth,
    #[error("Missing height")]
    MissingHeight,
    #[error("Invalid width")]
    InvalidWidth(ParseIntError),
    #[error("Invalid height")]
    InvalidHeight(ParseIntError),
    #[error("Invalid panel at {x}×{y}: {e}")]
    InvalidPanel {
        x: usize,
        y: usize,
        e: PanelParseError,
    },
    #[error("Premature end of input")]
    PrematureEndOfInput,
}

#[derive(Error, Debug, Clone)]
pub enum PanelParseError {
    #[error("Empty panel")]
    EmptyPanel,
    #[error("Invalid character {0}")]
    InvalidCharacter(char),
    #[error("Invalid color code {0}")]
    InvalidColor(char),
    #[error("Error parsing count: {0}")]
    InvalidCountStr(ParseIntError),
    #[error("Too many petals: {0}")]
    TooManyPetals(usize),
    #[error("Trailing bytes: {0:?}")]
    TrailingBytes(Vec<u8>),
}

impl FromStr for Grid {
    type Err = GridParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (size, rest) = s.split_once('\n').ok_or(GridParseError::EmptyInput)?;
        let rest = rest.trim();
        let mut sit = size.split_whitespace();
        let width = sit
            .next()
            .ok_or(GridParseError::MissingWidth)?
            .parse::<usize>()
            .map_err(GridParseError::InvalidWidth)?;
        let height = sit
            .next()
            .ok_or(GridParseError::MissingHeight)?
            .parse::<usize>()
            .map_err(GridParseError::InvalidHeight)?;

        let mut grid = Grid::new(width, height);

        let mut it = rest.split_whitespace();

        for y in 0..height {
            for (x, row) in grid.contents.iter_mut().enumerate() {
                let panel = &mut row[y];
                let token = it.next().ok_or(GridParseError::PrematureEndOfInput)?;
                *panel = token
                    .parse()
                    .map_err(|e| GridParseError::InvalidPanel { x, y, e })?;
            }
        }
        Ok(grid)
    }
}
impl FromStr for Panel {
    type Err = PanelParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut p = Panel::default();

        let rest = match s.as_bytes() {
            [f @ (b'.' | b'x' | b'X'), b'!', rest @ ..] => {
                p.filled = *f != b'.';
                p.fixed = true;
                rest
            }
            [f @ (b'.' | b'x' | b'X'), rest @ ..] => {
                p.filled = *f != b'.';
                rest
            }

            [c, ..] => return Err(PanelParseError::InvalidCharacter(*c as char)),
            [] => return Err(PanelParseError::EmptyPanel),
        };
        let rest = match rest {
            [l @ (b'-' | b'/'), c, rest @ ..] => {
                let color = panel::Color::from_code(*c as char)
                    .ok_or(PanelParseError::InvalidColor(*c as char))?;
                p.symbol = Symbol::Line {
                    diagonal: *l == b'/',
                    color,
                };
                rest
            }
            [b'o' | b'O', c, rest @ ..] => {
                let color = panel::Color::from_code(*c as char)
                    .ok_or(PanelParseError::InvalidColor(*c as char))?;
                p.symbol = Symbol::Lozange { color };
                rest
            }
            [b'c' | b'C', c, rest @ ..] => {
                let color = panel::Color::from_code(*c as char)
                    .ok_or(PanelParseError::InvalidColor(*c as char))?;
                // At this point, rest should still be valid UTF-8, since we know that color was a single-byte character
                let count =
                    std::str::from_utf8(rest).expect("Accidentally generated invalid UTF-8");
                let count = count
                    .parse::<i8>()
                    .map_err(PanelParseError::InvalidCountStr)?;
                p.symbol = Symbol::Pips { count, color };
                &[]
            }
            [b'f' | b'F', rest @ ..] => {
                let count =
                    std::str::from_utf8(rest).expect("Accidentally generated invalid UTF-8");
                let count = count
                    .parse::<usize>()
                    .map_err(PanelParseError::InvalidCountStr)?;
                if count > 4 {
                    return Err(PanelParseError::TooManyPetals(count));
                }
                p.symbol = Symbol::Petals { count };
                &[]
            }
            [c, ..] => return Err(PanelParseError::InvalidCharacter(*c as char)),
            [] => &[],
        };
        if !rest.is_empty() {
            Err(PanelParseError::TrailingBytes(rest.to_vec()))
        } else {
            Ok(p)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::panel::{Panel, COLORS, Symbol};

    #[test]
    fn parse1() {
        for filled in [false, true] {
            for fixed in [false, true] {
                let panel = Panel { filled, fixed, symbol: Symbol::Plain};
                assert_eq!(panel, panel.to_string().parse().unwrap());

                for color in COLORS {
                    let panel = Panel { filled, fixed, symbol: Symbol::Lozange { color }};
                    assert_eq!(panel, panel.to_string().parse().unwrap());
                }

                for count in 0..4 {
                    let panel = Panel { filled, fixed, symbol: Symbol::Petals { count }};
                    assert_eq!(panel, panel.to_string().parse().unwrap());
                }

                for diagonal in [false, true] {
                    for color in COLORS {
                        let panel = Panel { filled, fixed, symbol: Symbol::Line { diagonal, color }};
                        assert_eq!(panel, panel.to_string().parse().unwrap());
                    }
                }

                for color in COLORS {
                    for count in -5..0 {
                        let panel = Panel { filled, fixed, symbol: Symbol::Pips { count, color }};
                        assert_eq!(panel, panel.to_string().parse().unwrap());
                    }
                    for count in 1..10 {
                        let panel = Panel { filled, fixed, symbol: Symbol::Pips { count, color }};
                        assert_eq!(panel, panel.to_string().parse().unwrap());
                    }
                }
                
            }
        }
    }
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Grid {
            width,
            height,
            contents: vec![vec![Panel::default(); height]; width],
        }
    }
    pub fn neighbourhood(&self, x: usize, y: usize) -> Neighbourhood {
        let mut seen = HashSet::new();
        seen.insert((x, y));
        let mut to_visit = vec![(x, y)];
        let mut neighbour_panels = vec![];
        let target_filled = self.contents[x][y].filled;
        while let Some((vx, vy)) = to_visit.pop() {
            neighbour_panels.push((
                vx as i8 - x as i8,
                vy as i8 - y as i8,
                self.contents[vx][vy],
            ));

            if vx > 0
                && self.contents[vx - 1][vy].filled == target_filled
                && seen.insert((vx - 1, vy))
            {
                to_visit.push((vx - 1, vy));
            }
            if vy > 0
                && self.contents[vx][vy - 1].filled == target_filled
                && seen.insert((vx, vy - 1))
            {
                to_visit.push((vx, vy - 1));
            }
            if vx + 1 < self.width
                && self.contents[vx + 1][vy].filled == target_filled
                && seen.insert((vx + 1, vy))
            {
                to_visit.push((vx + 1, vy));
            }
            if vy + 1 < self.height
                && self.contents[vx][vy + 1].filled == target_filled
                && seen.insert((vx, vy + 1))
            {
                to_visit.push((vx, vy + 1));
            }
        }
        neighbour_panels.sort_unstable_by_key(|panel| (panel.0, panel.1));
        Neighbourhood {
            offset_x: x,
            offset_y: y,
            contents: neighbour_panels,
        }
    }

    pub fn reset(&mut self) {
        for (_, _, panel) in self.iter_mut() {
            if !panel.fixed {
                panel.filled = false;
            }
        }
    }

    pub fn is_solved(&self) -> bool {
        for (x, y, panel) in self.iter() {
            if panel.satisfied(x, y, self).is_err() {
                return false;
            }
        }
        true
    }
    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, &Panel)> {
        self.contents
            .iter()
            .enumerate()
            .flat_map(|(x, row)| row.iter().enumerate().map(move |(y, panel)| (x, y, panel)))
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, usize, &mut Panel)> {
        self.contents.iter_mut().enumerate().flat_map(|(x, row)| {
            row.iter_mut()
                .enumerate()
                .map(move |(y, panel)| (x, y, panel))
        })
    }
}
