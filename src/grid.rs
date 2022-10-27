use std::{
    collections::HashSet,
    fmt::{Display, Write},
    num::ParseIntError,
    str::FromStr,
};

use thiserror::Error;

use crate::panel::{self, Panel, Symbol};
mod neighbourhood;
mod solver;
pub use neighbourhood::*;

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
        let mut last_y = 0;
        for (_, y, panel) in self.iter() {
            if y != last_y {
                f.write_char('\n')?;
            }
            last_y = y;
            panel.fmt(f)?;
            f.write_char(' ')?;
        }
        f.write_char('\n')?;
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
        from_str_gridlike(s)
    }
}

fn from_str_gridlike<'a, G: Gridlike<'a>>(s: &str) -> Result<G, GridParseError> {
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

    let mut grid = G::new(width, height);

    let mut it = rest.split_whitespace();

    for y in 0..height {
        for x in 0..width {
            let token = it.next().ok_or(GridParseError::PrematureEndOfInput)?;
            let panel: Panel =
                token
                    .parse()
                    .map_err(|e| GridParseError::InvalidPanel { x, y, e })?;

            grid.set_lit_at(x, y, panel.filled);
            grid.set_fixed_at(x, y, panel.fixed);
            if panel.symbol != Symbol::Plain {
                grid.set_symbol_at(x, y, panel.symbol)
            }
        }
    }
    Ok(grid)
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
    use crate::panel::{Panel, Symbol, COLORS};

    #[test]
    fn parse1() {
        for filled in [false, true] {
            for fixed in [false, true] {
                let panel = Panel {
                    filled,
                    fixed,
                    symbol: Symbol::Plain,
                };
                assert_eq!(panel, panel.to_string().parse().unwrap());

                for color in COLORS {
                    let panel = Panel {
                        filled,
                        fixed,
                        symbol: Symbol::Lozange { color },
                    };
                    assert_eq!(panel, panel.to_string().parse().unwrap());
                }

                for count in 0..4 {
                    let panel = Panel {
                        filled,
                        fixed,
                        symbol: Symbol::Petals { count },
                    };
                    assert_eq!(panel, panel.to_string().parse().unwrap());
                }

                for diagonal in [false, true] {
                    for color in COLORS {
                        let panel = Panel {
                            filled,
                            fixed,
                            symbol: Symbol::Line { diagonal, color },
                        };
                        assert_eq!(panel, panel.to_string().parse().unwrap());
                    }
                }

                for color in COLORS {
                    for count in -5..0 {
                        let panel = Panel {
                            filled,
                            fixed,
                            symbol: Symbol::Pips { count, color },
                        };
                        assert_eq!(panel, panel.to_string().parse().unwrap());
                    }
                    for count in 1..10 {
                        let panel = Panel {
                            filled,
                            fixed,
                            symbol: Symbol::Pips { count, color },
                        };
                        assert_eq!(panel, panel.to_string().parse().unwrap());
                    }
                }
            }
        }
    }
}

pub trait Gridlike<'a> {
    type Iter: Iterator<Item = (usize, usize, Panel)> + 'a;
    /// Creates a new, empty grid with the requested dimensions
    fn new(width: usize, height: usize) -> Self;

    /// Returns the width of the grid
    fn width(&self) -> usize;
    
    /// Returns the height of the grid
    fn height(&self) -> usize;
    
    /// Returns panel information for the requested coordinates. Will panic if an out-of-bounds location is requested.
    fn panel_at(&self, x: usize, y: usize) -> Panel {
        Panel {
            filled: self.lit_at(x, y),
            fixed: self.fixed_at(x, y),
            symbol: self.symbol_at(x, y).unwrap_or(Symbol::Plain),
        }
    }

    /// Returns the symbol at the requested location, if any, or None if there is no symbol
    fn symbol_at(&self, x: usize, y: usize) -> Option<Symbol>;
    
    /// Returns a mutable reference the symbol at the requested location, if any, or None if there is no symbol
    fn symbol_at_mut(&mut self, x: usize, y: usize) -> Option<&mut Symbol>;

    /// Returns whether the panel at the requested location is lit. Panics if an out-of-bounds location is requested.
    fn lit_at(&self, x: usize, y: usize) -> bool;

    /// Toggles the lit status of the panel at the requested location. Panics if an out-of-bounds location is requested.
    fn toggle_lit_at(&mut self, x: usize, y: usize);

    /// Sets the lit status of the panel at the requested location. Panics if an out-of-bounds location is requested.
    fn set_lit_at(&mut self, x: usize, y: usize, lit: bool);

    fn colit_at(&self, _x: usize, _y: usize) -> bool {
        false
    }
    fn toggle_colit_at(&mut self, _x: usize, _y: usize) {}
    fn set_colit_at(&mut self, _x: usize, _y: usize, _lit: bool) {}

    fn fixed_at(&self, x: usize, y: usize) -> bool;
    fn toggle_fixed_at(&mut self, x: usize, y: usize);
    fn set_fixed_at(&mut self, x: usize, y: usize, lit: bool);
    fn neighbourhood(&self, x: usize, y: usize) -> Neighbourhood;
    fn neighbourhood_upto(&self, x: usize, y: usize, upto_x: usize, upto_y: usize)
        -> Neighbourhood;
    fn reset(&mut self);
    fn set_symbol_at(&mut self, x: usize, y: usize, symbol: Symbol);
    fn is_solved(&self) -> bool;
    fn rotate(&mut self);
    fn iter(&'a self) -> Self::Iter;
    fn solve(&self) -> Option<Self>
    where
        Self: Sized;
    fn symbols(&self) -> Vec<(usize, usize, Symbol)>;
}

const MAX_ROWS: usize = 12;
#[derive(Debug, Clone)]
pub struct Grid {
    width: usize,
    height: usize,
    lit: [u16; MAX_ROWS],
    colit: [u16; MAX_ROWS],
    fixed: [u16; MAX_ROWS],
    symbols: Vec<(usize, usize, Symbol)>,
}

impl<'a> Gridlike<'a> for Grid {
    fn colit_at(&self, x: usize, y: usize) -> bool {
        self.colit[y] & (1 << x) != 0
    }
    fn toggle_colit_at(&mut self, x: usize, y: usize) {
        self.colit[y] ^= 1 << x;
    }
    fn set_colit_at(&mut self, x: usize, y: usize, lit: bool) {
        if lit {
            self.colit[y] |= 1 << x;
            self.lit[y] &= !(1 << x);
        } else {
            self.colit[y] &= !(1 << x);
        }
    }

    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            lit: [0; MAX_ROWS],
            colit: [0; MAX_ROWS],
            fixed: [0; MAX_ROWS],
            symbols: vec![],
        }
    }
    fn symbols(&self) -> Vec<(usize, usize, Symbol)> {
        self.symbols.clone()
    }
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn symbol_at(&self, x: usize, y: usize) -> Option<Symbol> {
        self.symbols
            .iter()
            .find(|(sx, sy, _)| *sx as usize == x && *sy as usize == y)
            .map(|(_, _, s)| *s)
    }

    fn symbol_at_mut(&mut self, x: usize, y: usize) -> Option<&mut Symbol> {
        self.symbols
            .iter_mut()
            .find(|(sx, sy, _)| *sx as usize == x && *sy as usize == y)
            .map(|(_, _, s)| s)
    }

    fn lit_at(&self, x: usize, y: usize) -> bool {
        assert!(x < self.width && y < self.height);
        self.lit[y] & (1 << x) == (1 << x)
    }

    fn toggle_lit_at(&mut self, x: usize, y: usize) {
        assert!(x < self.width && y < self.height);
        self.lit[y] ^= 1 << x;
    }

    fn set_lit_at(&mut self, x: usize, y: usize, lit: bool) {
        if lit {
            self.lit[y] |= 1 << x;
        } else {
            self.lit[y] &= !(1 << x);
        }
    }

    fn fixed_at(&self, x: usize, y: usize) -> bool {
        assert!(x < self.width && y < self.height);
        self.fixed[y] & (1 << x) != 0
    }

    fn toggle_fixed_at(&mut self, x: usize, y: usize) {
        assert!(x < self.width && y < self.height);
        self.fixed[y] ^= 1 << x;
    }

    fn set_fixed_at(&mut self, x: usize, y: usize, fixed: bool) {
        if fixed {
            self.fixed[y] |= 1 << x;
        } else {
            self.fixed[y] &= !(1 << x);
        }
    }

    fn neighbourhood(&self, x: usize, y: usize) -> Neighbourhood {
        let mut seen = HashSet::new();
        seen.insert((x, y));
        let mut to_visit = vec![(x, y)];
        let mut neighbour_panels = vec![];
        let target_filled = self.lit_at(x, y);
        while let Some((vx, vy)) = to_visit.pop() {
            neighbour_panels.push((
                vx as i8 - x as i8,
                vy as i8 - y as i8,
                Panel {
                    filled: self.lit_at(vx, vy),
                    fixed: self.fixed_at(vx, vy),
                    symbol: self.symbol_at(vx, vy).unwrap_or_default(),
                },
            ));

            if vx > 0 && self.lit_at(vx - 1, vy) == target_filled && seen.insert((vx - 1, vy)) {
                to_visit.push((vx - 1, vy));
            }
            if vy > 0 && self.lit_at(vx, vy - 1) == target_filled && seen.insert((vx, vy - 1)) {
                to_visit.push((vx, vy - 1));
            }
            if vx + 1 < self.width
                && self.lit_at(vx + 1, vy) == target_filled
                && seen.insert((vx + 1, vy))
            {
                to_visit.push((vx + 1, vy));
            }
            if vy + 1 < self.height
                && self.lit_at(vx, vy + 1) == target_filled
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

    fn neighbourhood_upto(
        &self,
        x: usize,
        y: usize,
        upto_x: usize,
        upto_y: usize,
    ) -> Neighbourhood {
        if (y, x) > (upto_y, upto_x) {
            return Neighbourhood {
                offset_x: x,
                offset_y: y,
                contents: vec![],
            };
        }
        let mut seen = HashSet::new();
        seen.insert((x, y));
        let mut to_visit = vec![(x, y)];
        let mut neighbour_panels = vec![];
        let target_filled = self.lit_at(x, y);
        while let Some((vx, vy)) = to_visit.pop() {
            if (vy, vx) > (upto_y, upto_x) && !self.fixed_at(vx, vy) {
                continue;
            }
            neighbour_panels.push((
                vx as i8 - x as i8,
                vy as i8 - y as i8,
                Panel {
                    filled: self.lit_at(vx, vy),
                    fixed: self.fixed_at(vx, vy),
                    symbol: self.symbol_at(vx, vy).unwrap_or_default(),
                },
            ));

            if vx > 0 && self.lit_at(vx - 1, vy) == target_filled && seen.insert((vx - 1, vy)) {
                to_visit.push((vx - 1, vy));
            }
            if vy > 0 && self.lit_at(vx, vy - 1) == target_filled && seen.insert((vx, vy - 1)) {
                to_visit.push((vx, vy - 1));
            }
            if vx + 1 < self.width
                && self.lit_at(vx + 1, vy) == target_filled
                && seen.insert((vx + 1, vy))
            {
                to_visit.push((vx + 1, vy));
            }
            if vy + 1 < self.height
                && self.lit_at(vx, vy + 1) == target_filled
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

    fn reset(&mut self) {
        let mut rv = self.clone();
        for (x, y, panel) in self.iter() {
            if !panel.fixed {
                rv.set_lit_at(x, y, false);
            }
        }
        *self = rv;
    }

    fn set_symbol_at(&mut self, x: usize, y: usize, symbol: Symbol) {
        if symbol == Symbol::Plain {
            self.symbols.retain(|(xx, yy, _)| *xx != x || *yy != y);
        }
        if let Some(s) = self.symbol_at_mut(x, y) {
            *s = symbol;
        } else {
            self.symbols.push((x, y, symbol));
        }
    }

    fn is_solved(&self) -> bool {
        for (x, y, panel) in self.iter() {
            if panel.satisfied(x, y, self).is_err() {
                return false;
            }
        }
        true
    }

    fn rotate(&mut self) {
        let mut rv = Self::new(self.width, self.height);
        let rot = if self.width == self.height {
            Rotation::D90
        } else {
            Rotation::D180
        };
        for (x, y, panel) in self.iter() {
            let (nx, ny) = match rot {
                Rotation::D0 => (x, y),
                Rotation::D90 => (self.width - 1 - y, x),
                Rotation::D180 => (self.width - 1 - x, self.height - 1 - y),
                Rotation::D270 => (y, self.width - 1 - x),
            };
            if panel.filled {
                rv.toggle_lit_at(nx, ny);
            }
            if panel.fixed {
                rv.toggle_fixed_at(nx, ny);
            }
            if panel.symbol != Symbol::Plain {
                rv.set_symbol_at(nx, ny, panel.symbol);
            }
        }
        *self = rv;
    }
    type Iter = GridIter<'a>;
    fn iter(&'a self) -> Self::Iter {
        GridIter {
            x: 0,
            y: 0,
            grid: self,
        }
    }

    fn solve(&self) -> Option<Grid> {
        let mut rv = self.clone();
        rv.reset();
        if rv.is_solved() {
            return Some(rv);
        } else {
            rv.solve_from(0, 0)
        }
    }
}

pub struct GridIter<'a> {
    x: usize,
    y: usize,
    grid: &'a Grid,
}

impl<'a> Iterator for GridIter<'a> {
    type Item = (usize, usize, Panel);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= self.grid.width {
            self.x = 0;
            self.y += 1;
        }
        if self.y >= self.grid.height {
            return None;
        }

        let filled = self.grid.lit_at(self.x, self.y);
        let fixed = self.grid.fixed_at(self.x, self.y);
        let symbol = self.grid.symbol_at(self.x, self.y).unwrap_or(Symbol::Plain);
        let rv = (
            self.x,
            self.y,
            Panel {
                filled,
                fixed,
                symbol,
            },
        );
        self.x += 1;
        Some(rv)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PanelLoc {
    pub x: usize,
    pub y: usize,
}

impl Display for PanelLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.x + 1, ('A' as u8 + self.y as u8) as char)
    }
}

impl FromStr for PanelLoc {
    type Err = PanelLocParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numeric = s.trim_end_matches(|c: char| c.is_ascii_alphabetic());
        let character = s.trim_start_matches(|c: char| c.is_ascii_digit());
        if numeric.is_empty() {
            return Err(PanelLocParseError::Illformed);
        }
        if character.len() != 1 || !(b'A'..=b'Z').contains(&character.as_bytes()[0]) {
            return Err(PanelLocParseError::Illformed);
        }
        let x = numeric
            .parse::<usize>()
            .map_err(PanelLocParseError::CouldntParseWidth)?
            - 1;
        let y = (character.as_bytes()[0] - b'A') as usize;
        Ok(PanelLoc { x, y })
    }
}

#[derive(Debug, Error)]
pub enum PanelLocParseError {
    #[error("PanelLoc was ill-formed")]
    Illformed,
    #[error("Couldn't parse width: {0}")]
    CouldntParseWidth(ParseIntError),
}
