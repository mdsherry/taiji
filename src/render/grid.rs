use std::collections::HashSet;

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, StatefulWidget, Widget},
};

use crate::{
    grid::PanelLoc,
    panel::{self, Panel, Symbol},
    Gridlike, ROTATIONS,
};

pub struct GridRenderer<'a, G> {
    grid: &'a G,
}

impl<'a, G> GridRenderer<'a, G>
where
    G: Gridlike<'a>,
{
    pub fn new(grid: &'a G) -> Self {
        GridRenderer { grid }
    }
}

impl<'a, G: Gridlike<'a>> StatefulWidget for GridRenderer<'a, G> {
    fn render(self, area: Rect, buf: &mut Buffer, (cx, cy, tagged, error_locs): &mut Self::State) {
        if area.area() == 0 {
            return;
        }

        let panels_visible_wide = (area.width - 1) / PANEL_WIDTH;
        let panels_visible_high = (area.height - 1) / PANEL_WIDTH;
        let start_x = if panels_visible_wide < self.grid.width() as u16 {
            cx.saturating_sub((panels_visible_wide / 2) as usize)
        } else {
            0
        };

        let start_y = if panels_visible_high < self.grid.height() as u16 {
            cy.saturating_sub((panels_visible_high / 2) as usize)
        } else {
            0
        };
        let cotagged = get_cotagged(self.grid, tagged);
        for x in (start_x as u16)..self.grid.width() as u16 {
            if x - start_x as u16 >= panels_visible_wide {
                break;
            }
            buf.get_mut(
                1 + (x - start_x as u16) * PANEL_WIDTH + area.left() + PANEL_WIDTH / 2,
                area.top(),
            )
            .symbol = (x + 1).to_string();
        }

        for y in (start_y as u16)..self.grid.height() as u16 {
            if y - start_y as u16 >= panels_visible_high {
                break;
            }
            buf.get_mut(
                area.left(),
                1 + (y - start_y as u16) * PANEL_HEIGHT + area.top() + PANEL_HEIGHT / 2,
            )
            .symbol = ((b'A' + y as u8) as char).to_string();
        }

        for (x, y, panel) in self.grid.iter() {
            if x < start_x || y < start_y {
                continue;
            }
            let rect = Rect {
                x: (x - start_x) as u16 * PANEL_WIDTH + 2,
                y: (y - start_y) as u16 * PANEL_HEIGHT + 2,
                width: PANEL_WIDTH,
                height: PANEL_HEIGHT,
            };
            if rect_intersect(rect, area) == rect {
                render_panel(
                    x == *cx && y == *cy,
                    error_locs.contains(&PanelLoc { x, y }),
                    tagged.contains(&(x, y)),
                    cotagged.contains(&(x, y)),
                    self.grid.colit_at(x, y),
                    panel,
                    rect,
                    buf,
                )
            }
        }
    }

    type State = (usize, usize, HashSet<(usize, usize)>, HashSet<PanelLoc>);
}

fn get_cotagged<'a, G: Gridlike<'a>>(
    grid: &'a G,
    tagged: &mut HashSet<(usize, usize)>,
) -> Vec<(usize, usize)> {
    let mut cotagged = vec![];
    for &(tx, ty) in tagged.iter() {
        let tagged_neighbourhood = grid.neighbourhood(tx, ty);
        for (lx, ly, panel) in tagged_neighbourhood.grid_iter() {
            if let Symbol::Line { diagonal, color } = panel.symbol {
                // Find every other line of the same colour, and tag the corresponding location
                let l_neigh = grid.neighbourhood(lx, ly);
                let rx = tx as i32 - lx as i32;
                let ry = ty as i32 - ly as i32;
                for (xx, yy, opanel) in grid.iter() {
                    if xx == lx && yy == ly {
                        continue;
                    }
                    if let Symbol::Line {
                        diagonal: odiagonal,
                        color: ocolor,
                    } = opanel.symbol
                    {
                        if color != ocolor {
                            continue;
                        }
                        if diagonal || odiagonal {
                            // Find the most likely rotation
                            let oneigh = grid.neighbourhood(xx, yy);
                            let rot = ROTATIONS
                                .into_iter()
                                .max_by_key(|rot| l_neigh.rotated_overlap(&oneigh, *rot))
                                .unwrap();

                            let (rx, ry) = rot.rotate((rx as i8, ry as i8));
                            let olx = xx as i32 + rx as i32;
                            let oly = yy as i32 + ry as i32;
                            if olx >= 0
                                && olx < grid.width() as i32
                                && oly >= 0
                                && oly < grid.height() as i32
                            {
                                cotagged.push((olx as usize, oly as usize));
                            }
                        } else {
                            let olx = xx as i32 + rx;
                            let oly = yy as i32 + ry;
                            if olx >= 0
                                && olx < grid.width() as i32
                                && oly >= 0
                                && oly < grid.height() as i32
                            {
                                cotagged.push((olx as usize, oly as usize));
                            }
                        }
                    }
                }
            }
        }
    }
    cotagged
}

const PANEL_WIDTH: u16 = 5;
const PANEL_HEIGHT: u16 = 5;

struct PanelColours {
    border_bg: Color,
    border_fg: Color,
    bg: Color,
    fg: Color,
}

fn get_colours(
    cursor: bool,
    error: bool,
    _tagged: bool,
    _cotagged: bool,
    filled: bool,
) -> PanelColours {
    let border_bg = match (filled, cursor) {
        (true, true) => Color::DarkGray,
        (true, false) => Color::DarkGray,
        (false, true) => Color::Black,
        (false, false) => Color::Black,
    };
    let border_fg = match (error, cursor, filled) {
        (true, _, _) => Color::Red,
        (_, true, true) => Color::Black,
        (_, true, false) => Color::White,
        (_, false, true) => Color::Gray,
        (_, false, false) => Color::DarkGray,
    };
    PanelColours {
        border_bg,
        border_fg,
        bg: border_bg,
        fg: border_fg,
    }
}

fn render_panel(
    cursor: bool,
    error: bool,
    tagged: bool,
    cotagged: bool,
    colit: bool,
    panel: Panel,
    area: Rect,
    buf: &mut Buffer,
) {
    let mut colours = get_colours(cursor, error, tagged, cotagged, panel.filled);
    if colit {
        colours.border_bg = Color::Green;
        if colours.border_fg == Color::DarkGray {
            colours.border_fg = Color::Black;
        }
    }
    let block = Block::default()
        .style(
            Style::default()
                .bg(colours.bg)
                .add_modifier(if cursor {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                })
                .fg(colours.fg),
        )
        .borders(Borders::ALL)
        .border_type(if panel.fixed {
            BorderType::Double
        } else {
            BorderType::Plain
        })
        .border_style(
            Style::default()
                .fg(colours.border_fg)
                .bg(colours.border_bg)
                .add_modifier(if cursor {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        );
    block.render(area, buf);

    if cotagged {
        buf.get_mut(area.left() + PANEL_WIDTH / 2, area.top())
            .set_char('v');
        buf.get_mut(area.left() + PANEL_WIDTH / 2 - 1, area.top())
            .set_char('v');
        buf.get_mut(area.left() + PANEL_WIDTH / 2 + 1, area.top())
            .set_char('v');
    }
    if tagged {
        buf.get_mut(area.left() + PANEL_WIDTH / 2, area.top())
            .set_char('V');
        buf.get_mut(area.left() + PANEL_WIDTH / 2 - 1, area.top())
            .set_char('V');
        buf.get_mut(area.left() + PANEL_WIDTH / 2 + 1, area.top())
            .set_char('V');
    }
    let cell = buf.get_mut(area.left() + PANEL_WIDTH / 2, area.top() + PANEL_HEIGHT / 2);

    match panel.symbol {
        panel::Symbol::Plain => {
            cell.set_char('.');
        }
        panel::Symbol::Pips { count, color } => {
            cell.symbol = count.to_string();
            cell.fg = color.to_tui();
        }
        panel::Symbol::Line { diagonal, color } => {
            if diagonal {
                cell.set_char('╱');
            } else {
                cell.set_char('─');
            }
            cell.fg = color.to_tui();
        }
        panel::Symbol::Lozange { color } => {
            cell.set_char('⧫');
            cell.fg = color.to_tui();
        }
        panel::Symbol::Petals { count } => {
            cell.set_char(match count {
                0 => '🌑',
                1 => '🌒',
                2 => '🌓',
                3 => '🌔',
                4 => '🌕',
                _ => unreachable!(),
            });
        }
    }

    if cursor {
        buf.get_mut(area.left() + PANEL_WIDTH / 2, area.top() + PANEL_HEIGHT - 1)
            .set_char('^');
    }
}

fn rect_intersect(a: Rect, b: Rect) -> Rect {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2: u16 = (a.x + a.width).min(b.x + b.width);
    let y2: u16 = (a.y + a.height).min(b.y + b.height);
    Rect {
        x: x1,
        y: y1,
        width: x2.saturating_sub(x1),
        height: y2.saturating_sub(y1),
    }
}
