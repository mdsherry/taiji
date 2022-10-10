use crate::{Gridlike, grid::PanelLoc};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Color {
    #[default]
    Black,
    Yellow,
    Purple,
    White,
    Blue,
    Green,
}

#[allow(dead_code)]
pub const COLORS: [Color; 6] = [Color::Black, Color::Yellow, Color::Purple, Color::White, Color:: Blue, Color::Green];

impl Color {
    pub fn code(self) -> char {
        match self {
            Color::Black => 'B',
            Color::Yellow => 'Y',
            Color::Purple => 'P',
            Color::White => 'W',
            Color::Blue => 'U',
            Color::Green => 'G',
        }
    }
    pub fn from_code(ch: char) -> Option<Self> {
        Some(match ch {
            'b' | 'B' => Color::Black,
            'y' | 'Y' => Color::Yellow,
            'p' | 'P' => Color::Purple,
            'w' | 'W' => Color::White,
            'u' | 'U' => Color::Blue,
            'g' | 'G' => Color::Green,
            _ => return None
        })
    }
    pub fn to_tui(self) -> tui::style::Color {
        match self {
            Color::Black => tui::style::Color::Blue,
            Color::Yellow => tui::style::Color::Yellow,
            Color::Purple => tui::style::Color::LightMagenta,
            Color::White => tui::style::Color::White,
            Color::Blue => tui::style::Color::LightCyan,
            Color::Green => tui::style::Color::Green,
        }
    }
    pub fn next(self) -> Self {
        match self {
            Color::Black => Color::Yellow,
            Color::Yellow => Color::Purple,
            Color::Purple => Color::White,
            Color::White => Color::Blue,
            Color::Blue => Color::Green,
            Color::Green => Color::Black,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            Color::Black => Color::Green,
            Color::Yellow => Color::Black,
            Color::Purple => Color::Yellow,
            Color::White => Color::Purple,
            Color::Blue => Color::White,
            Color::Green => Color::Blue,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Symbol {
    #[default]
    Plain,
    Pips {
        count: i8,
        color: Color,
    },
    Line {
        diagonal: bool,
        color: Color,
    },
    Lozange {
        color: Color,
    },
    Petals {
        count: usize,
    },
}

impl Symbol {
    pub fn colour(&self) -> Option<Color> {
        match self {
            Symbol::Plain | Symbol::Petals {..} => None,
            Symbol::Pips { color, .. } | 
            Symbol::Line { color, .. } |
            Symbol::Lozange { color } => Some(*color)
        }
    }

    pub fn set_count(&mut self, c: i8) {
        match self {
            Symbol::Plain | Symbol::Line { .. } | Symbol::Lozange { .. }=> (),
            Symbol::Pips { count, .. } => *count = c,
            Symbol::Petals { count } => *count = c.min(4).max(0) as usize,
        }
    }

    pub fn incr_count(&mut self) {
        match self {
            Symbol::Plain | Symbol::Line { .. } | Symbol::Lozange { .. }=> (),
            Symbol::Pips { count, .. } => if *count == -1 { *count = 1; } else { *count += 1; },
            Symbol::Petals { count } => *count = (*count + 1).min(4) as usize,
        }
    }

    pub fn decr_count(&mut self) {
        match self {
            Symbol::Plain | Symbol::Line { .. } | Symbol::Lozange { .. }=> (),
            Symbol::Pips { count, .. } => if *count == 1 { *count = -1; } else { *count -= 1 },
            Symbol::Petals { count } => *count = (*count as i32 - 1).max(0) as usize,
        }
    }
}

#[derive(Clone, Debug, Copy, Default, PartialEq, Eq)]
pub struct Panel {
    pub filled: bool,
    pub fixed: bool,
    pub symbol: Symbol,
}

impl Panel {
    pub fn satisfiable<'a, G: Gridlike<'a>>(self, x: usize, y: usize, upto_x: usize, upto_y: usize, grid: &'a G) -> bool {
        if (y, x) >= (upto_y, upto_x) {
            return true;
        }
        match self.symbol {
            Symbol::Plain => true,
            Symbol::Pips { count: _, color } => {
                let neighbourhood = grid.neighbourhood(x, y);
                if !neighbourhood.entirely_before(upto_x, upto_y) {
                    return true;
                }
                let total_pip_sum: i8 = grid.iter().filter_map(|(_, _, panel)| {
                    if let Symbol::Pips { count, color: pcolor } = panel.symbol { 
                        if color == pcolor { 
                            Some(count)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }).sum();
                if total_pip_sum != 0 && grid.lit_at(x, y) && total_pip_sum < neighbourhood.contents.len() as i8 {
                    return false;
                }
                
                // let mut pip_sum = 0;
                // for (ox, oy, other_panel) in neighbourhood.contents.iter() {
                //     if let Symbol::Pips {
                //         color: other_color,
                //         count,
                //     } = other_panel.symbol
                //     {
                //         if color != other_color {
                //             return false;
                //         }
                //         pip_sum += count;
                //     }
                // }
                true
            }
            Symbol::Line { .. } => {
                true
            },
            Symbol::Lozange { color } => {
                let neighbourhood = grid.neighbourhood(x, y);
                if !neighbourhood.entirely_before(upto_x, upto_y) {
                    return true;
                }
                let mut lozange_count = 0;
                for (_ox, _oy, other_panel) in neighbourhood.contents.iter() {
                    match other_panel.symbol {
                        Symbol::Pips { color: other_color, .. } if color == other_color => lozange_count += 1,
                        Symbol::Line { color: other_color, ..} if color == other_color => lozange_count += 1,
                        Symbol::Lozange { color: other_color, ..} if color == other_color => lozange_count += 1,
                        Symbol::Petals { count: petal_count } => {
                            if color == Color::Yellow && petal_count != 0 {
                                lozange_count += 1;
                            } else if color == Color::Purple && petal_count != 4 {
                                lozange_count += 1;
                            }
                        },
                        _ => ()
                    }
                }
                
                lozange_count <= 2
            }
            Symbol::Petals { count } => {
                if y + 1 < grid.height() {
                    if upto_y < y + 1 || upto_x < x {
                        return true
                    }
                } else if x + 1 < grid.width() {
                    if upto_y < y || upto_x < x + 1 {
                        return true;
                    }
                } else {
                    if upto_y < y || upto_x < x {
                        return true;
                    }
                }
                let mut neighbour_count = 0;
                if x > 0 && grid.lit_at(x - 1, y) == self.filled {
                    neighbour_count += 1;
                }
                if x + 1 < grid.width() && grid.lit_at(x + 1, y) == self.filled {
                    neighbour_count += 1;
                }
                if y > 0 && grid.lit_at(x, y - 1) == self.filled {
                    neighbour_count += 1;
                }
                if y + 1 < grid.height() && grid.lit_at(x, y + 1) == self.filled {
                    neighbour_count += 1;
                }

                if neighbour_count == count {
                    true
                } else {
                    false
                }
            },
        }
    }

    pub fn satisfied<'a>(self, x: usize, y: usize, grid: &'a impl Gridlike<'a>) -> Result<(), PanelError> {
        match self.symbol {
            Symbol::Plain => Ok(()),
            Symbol::Pips { count: _, color } => {
                let neighbourhood = grid.neighbourhood(x, y);
                let mut pip_sum = 0;
                for (ox, oy, other_panel) in neighbourhood.contents.iter() {
                    if let Symbol::Pips {
                        color: other_color,
                        count,
                    } = other_panel.symbol
                    {
                        if color != other_color {
                            return Err(PanelError::OverlappingPips {
                                pos: PanelLoc { x, y }, 
                                other_pos: PanelLoc {x: (x as i8 + ox) as _, y: (y as i8 + oy) as _ },
                                color,
                                other_color,
                            });
                        }
                        pip_sum += count;
                    }
                }
                let size = neighbourhood.contents.len();
                if pip_sum == 0 {
                    Ok(())
                } else if size as i8 != pip_sum {
                    return Err(PanelError::WrongNeighbourhoodSize {
                        pos: PanelLoc { x, y },
                        required: pip_sum,
                        have: size as i8,
                    });
                } else {
                    Ok(())
                }
            }
            Symbol::Line { diagonal, color } => {
                let my_neighbourhood = grid.neighbourhood(x, y);
                
                // Get all other neighbourhoods of the same color
                for (cx, cy, panel) in grid.iter() {
                    if cx == x && cy == y {
                        continue;
                    }
                    if let Symbol::Line { diagonal: other_diagonal, color: other_color } = panel.symbol {
                        let other_neighbourhood = grid.neighbourhood(cx, cy);
                        let shape_matches = if diagonal || other_diagonal {
                            my_neighbourhood.same_shape_rotated(&other_neighbourhood)
                        } else {
                             my_neighbourhood.same_shape(&other_neighbourhood)
                        };
                        if color == other_color {
                            if !shape_matches {
                                return Err(PanelError::LineNeighbourhoodWrongShape {
                                    pos: PanelLoc { x, y }, other_pos: PanelLoc {x: cx, y: cy }
                                });
                            }
                        } else {
                            if shape_matches {
                                return Err(PanelError::DuplicateLineNeighbourhoodShape {
                                    pos: PanelLoc { x, y }, other_pos: PanelLoc {x: cx, y: cy }
                                });
                            }
                        }
                    }
                }
            
                Ok(())
            },
            Symbol::Lozange { color } => {
                let neighbourhood = grid.neighbourhood(x, y);
                let mut lozange_count = 0;
                for (_ox, _oy, other_panel) in neighbourhood.contents.iter() {
                    
                    match other_panel.symbol {
                        Symbol::Petals { count: petal_count } => {
                            if color == Color::Yellow && petal_count != 0{
                                lozange_count += 1;
                            } else if color == Color::Purple && petal_count != 4{
                                lozange_count += 1;
                            }
                        },
                        _ => if other_panel.symbol.colour() == Some(color) { lozange_count += 1 }
                    }
                }
                if lozange_count != 2 {
                    Err(PanelError::LozangeCount { pos: PanelLoc { x, y }, saw: lozange_count as i32, color })
                } else {
                    Ok(())
                }
            }
            Symbol::Petals { count } => {
                let mut neighbour_count = 0;
                if x > 0 && grid.lit_at(x - 1, y) == self.filled {
                    neighbour_count += 1;
                }
                if x + 1 < grid.width() && grid.lit_at(x + 1, y) == self.filled {
                    neighbour_count += 1;
                }
                if y > 0 && grid.lit_at(x, y - 1) == self.filled {
                    neighbour_count += 1;
                }
                if y + 1 < grid.height() && grid.lit_at(x, y + 1) == self.filled {
                    neighbour_count += 1;
                }

                if neighbour_count == count {
                    Ok(())
                } else {
                    Err(PanelError::PetalCount { pos: PanelLoc { x, y }, saw: neighbour_count, required: count })
                }
            },
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum PanelError {
    #[error(
        "Pips at {pos} conflict in color with pips at {other_pos} ({color:?} vs {other_color:?})"
    )]
    OverlappingPips {
        pos: PanelLoc,
        other_pos: PanelLoc,
        color: Color,
        other_color: Color,
    },
    #[error("Neighbourhood of pips at {pos} must be of size {required} but was of size {have}")]
    WrongNeighbourhoodSize {
        pos: PanelLoc,
        required: i8,
        have: i8,
    },
    #[error("Flower has wrong number of petals at {pos}: must have {required} but had {saw}")]
    PetalCount {
        pos: PanelLoc,
        required: usize,
        saw: usize,
    },
    #[error("Saw {saw} {color:?} lozanges in the neighbourhood of {pos}, instead of 2")]
    LozangeCount {
        pos: PanelLoc,
        saw: i32,
        color: Color,
    },
    #[error("Neighbourhood for line at {pos} was wrong shape. Conflicting line at {other_pos}")]
    LineNeighbourhoodWrongShape {
        pos: PanelLoc,
        other_pos: PanelLoc,
    },
    #[error("Neighbourhood for line at {pos} matches line of a different colour at {other_pos}")]
    DuplicateLineNeighbourhoodShape {
        pos: PanelLoc,
        other_pos: PanelLoc,
    },
}

impl PanelError {
    pub fn get_pos(&self) -> PanelLoc {
        match self {
            PanelError::OverlappingPips { pos, .. } => *pos,
            PanelError::WrongNeighbourhoodSize { pos, .. } => *pos,
            PanelError::PetalCount { pos, .. } => *pos,
            PanelError::LozangeCount { pos, .. } => *pos,
            PanelError::LineNeighbourhoodWrongShape { pos, .. } => *pos,
            PanelError::DuplicateLineNeighbourhoodShape { pos, .. } => *pos,
        }
    }
}