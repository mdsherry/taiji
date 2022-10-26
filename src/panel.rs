use crate::{
    grid::{Neighbourhood, PanelLoc},
    Gridlike,
};
use thiserror::Error;

pub use crate::symbol::*;

#[derive(Clone, Debug, Copy, Default, PartialEq, Eq)]
pub struct Panel {
    pub filled: bool,
    pub fixed: bool,
    pub symbol: Symbol,
}

impl Panel {
    pub fn satisfiable<'a, G: Gridlike<'a>>(
        self,
        x: usize,
        y: usize,
        upto_x: usize,
        upto_y: usize,
        grid: &'a G,
    ) -> bool {
        if (y, x) > (upto_y, upto_x) {
            return true;
        }
        match self.symbol {
            Symbol::Plain => true,
            Symbol::Pips { count: _, color } => {
                let neighbourhood = grid.neighbourhood_upto(x, y, upto_x, upto_y);

                let mut pip_sum = 0;
                for (_, _, other_panel) in neighbourhood.contents.iter() {
                    if let Symbol::Pips {
                        color: other_color,
                        count,
                    } = other_panel.symbol
                    {
                        if color != other_color {
                            return false;
                        }
                        pip_sum += count;
                    }
                }
                pip_sum != 0 && pip_sum >= neighbourhood.contents.len() as i8
            }
            Symbol::Line { diagonal, color } => {
                let mut neighbourhood = grid.neighbourhood_upto(x, y, upto_x, upto_y);
                for (sx, sy, symbol) in grid.symbols() {
                    if let Symbol::Line {
                        diagonal: odiagonal,
                        color: ocolor,
                    } = symbol
                    {
                        if color != ocolor {
                            continue;
                        }
                        neighbourhood.translate_to(sx, sy);
                        let d_inbounds = (diagonal || odiagonal)
                            && neighbourhood.inbounds_rotated(grid.width(), grid.height());
                        if !(d_inbounds || neighbourhood.inbounds(grid.width(), grid.height())) {
                            return false;
                        }
                    }
                }
                true
            }
            Symbol::Lozange { color } => {
                let neighbourhood = grid.neighbourhood_upto(x, y, upto_x, upto_y);

                let mut lozange_count = 0;
                for (_ox, _oy, other_panel) in neighbourhood.contents.iter() {
                    match other_panel.symbol {
                        Symbol::Pips {
                            color: other_color, ..
                        } if color == other_color => lozange_count += 1,
                        Symbol::Line {
                            color: other_color, ..
                        } if color == other_color => lozange_count += 1,
                        Symbol::Lozange {
                            color: other_color, ..
                        } if color == other_color => lozange_count += 1,
                        Symbol::Petals { count: petal_count } => {
                            if color == Color::Yellow && petal_count != 0 {
                                lozange_count += 1;
                            } else if color == Color::Purple && petal_count != 4 {
                                lozange_count += 1;
                            }
                        }
                        _ => (),
                    }
                }

                lozange_count <= 2
            }
            Symbol::Petals { count } => {
                let mut check_area = Neighbourhood::new_around(x, y, grid);
                check_area.constrain_to_before(upto_x, upto_y);
                let mut definitely_on = 0;
                let mut definitely_off = 0;
                for (x, y, panel) in check_area.contents.iter() {
                    if *x == 0 && *y == 0 {
                        continue;
                    }
                    if panel.filled == self.filled {
                        definitely_on += 1;
                    } else {
                        definitely_off += 1;
                    }
                }

                count >= definitely_on && 4 - count >= definitely_off
            }
        }
    }

    pub fn satisfied<'a>(
        self,
        x: usize,
        y: usize,
        grid: &'a impl Gridlike<'a>,
    ) -> Result<(), PanelError> {
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
                                other_pos: PanelLoc {
                                    x: (x as i8 + ox) as _,
                                    y: (y as i8 + oy) as _,
                                },
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
                    if let Symbol::Line {
                        diagonal: other_diagonal,
                        color: other_color,
                    } = panel.symbol
                    {
                        let other_neighbourhood = grid.neighbourhood(cx, cy);
                        let shape_matches = if diagonal || other_diagonal {
                            my_neighbourhood.same_shape_rotated(&other_neighbourhood)
                        } else {
                            my_neighbourhood.same_shape(&other_neighbourhood)
                        };
                        if color == other_color {
                            if !shape_matches {
                                return Err(PanelError::LineNeighbourhoodWrongShape {
                                    pos: PanelLoc { x, y },
                                    other_pos: PanelLoc { x: cx, y: cy },
                                });
                            }
                        } else {
                            if shape_matches {
                                return Err(PanelError::DuplicateLineNeighbourhoodShape {
                                    pos: PanelLoc { x, y },
                                    other_pos: PanelLoc { x: cx, y: cy },
                                });
                            }
                        }
                    }
                }

                Ok(())
            }
            Symbol::Lozange { color } => {
                let neighbourhood = grid.neighbourhood(x, y);
                let mut lozange_count = 0;
                for (_, _, other_panel) in neighbourhood.contents.iter() {
                    match other_panel.symbol {
                        Symbol::Petals { count: petal_count } => {
                            if color == Color::Yellow && petal_count != 0 {
                                lozange_count += 1;
                            } else if color == Color::Purple && petal_count != 4 {
                                lozange_count += 1;
                            }
                        }
                        _ => {
                            if other_panel.symbol.colour() == Some(color) {
                                lozange_count += 1
                            }
                        }
                    }
                }
                if lozange_count != 2 {
                    Err(PanelError::LozangeCount {
                        pos: PanelLoc { x, y },
                        saw: lozange_count as i32,
                        color,
                    })
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
                    Err(PanelError::PetalCount {
                        pos: PanelLoc { x, y },
                        saw: neighbour_count,
                        required: count,
                    })
                }
            }
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
    LineNeighbourhoodWrongShape { pos: PanelLoc, other_pos: PanelLoc },
    #[error("Neighbourhood for line at {pos} matches line of a different colour at {other_pos}")]
    DuplicateLineNeighbourhoodShape { pos: PanelLoc, other_pos: PanelLoc },
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
