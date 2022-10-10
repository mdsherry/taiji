use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
};

use std::{io, collections::{HashSet, HashMap}};
use tui::{
    backend::{Backend},
    Terminal,
};

use crate::{grid::{Grid, Rotation}, panel::{self, Symbol}, Args, render::ui};

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, args: Args) -> io::Result<()> {
    let mut grid = if let Some(in_path) = args.in_file {
        let s = std::fs::read_to_string(in_path)?;
        s.parse().unwrap()
    } else {
        Grid::new(args.width, args.height)
    };
    let mut cx = 0;
    let mut cy = 0;
    let mut tagged = HashSet::new();
    let mut cur_color = panel::Color::Yellow;
    let mut state_stack = vec![];
    loop {
        terminal.draw(|f| ui(f, &mut grid, cx, cy, cur_color, state_stack.len(), tagged.clone()))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::PageUp => {
                    state_stack.push(grid.clone());
                }
                KeyCode::PageDown => {
                    if !state_stack.is_empty() {
                        grid = state_stack.pop().unwrap();
                    }
                }
                KeyCode::Enter => {
                    if !tagged.insert((cx, cy)) {
                        tagged.remove(&(cx, cy));
                    }
                }
                KeyCode::Char('S') => {
                    if let Some(solution) = grid.solve() {
                        grid = solution;
                    }
                }
                KeyCode::Char(n@'1'..='9') => {
                    let n = (n as u8 - b'1' + 1) as usize;
                    match &mut grid.contents[cx][cy].symbol {
                        Symbol::Plain | Symbol::Line {..} | Symbol::Lozange {..} => (),
                        Symbol::Pips { count, ..} => *count = n as i8,
                        Symbol::Petals { count } => *count = n.min(4),
                    }
                }
                KeyCode::Tab => {
                    let mut rv = grid.clone();
                    let rot = if grid.width == grid.height { Rotation::D90 } else { Rotation::D180 };
                    for (x, y, panel) in grid.iter() {
                        let (nx, ny) = match rot {
                            Rotation::D0 => (x, y),
                            Rotation::D90 => (grid.width - 1 - y, x),
                            Rotation::D180 => (grid.width - 1 - x, grid.height - 1 - y),
                            Rotation::D270 => (y, grid.width - 1 - x),
                        };
                        rv.contents[nx as usize][ny as usize] = *panel;
                    }
                    grid = rv;
                }
                KeyCode::Up => {
                    if cy > 0 {
                        cy -= 1;
                    }
                }
                KeyCode::Down => {
                    if cy + 1 < grid.height {
                        cy += 1;
                    }
                }
                KeyCode::Left => {
                    if cx > 0 {
                        cx -= 1;
                    }
                }
                KeyCode::Right => {
                    if cx + 1 < grid.width {
                        cx += 1;
                    }
                }
                KeyCode::Char(' ') => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        grid.contents[cx][cy].fixed = !grid.contents[cx][cy].fixed;
                    } else {
                        if !grid.contents[cx][cy].fixed {
                            grid.contents[cx][cy].filled = !grid.contents[cx][cy].filled;
                        }
                    }
                }
                KeyCode::Char('r') => {
                    for row in &mut grid.contents {
                        for panel in row {
                            if !panel.fixed {
                                panel.filled = false;
                            }
                        }
                    }
                }
                KeyCode::Char('.') => {
                    grid.contents[cx][cy].symbol = Symbol::Plain;
                }
                KeyCode::Char('l') => {
                    grid.contents[cx][cy].symbol = Symbol::Line {
                        diagonal: false,
                        color: cur_color,
                    };
                }
                KeyCode::Char('L') => {
                    grid.contents[cx][cy].symbol = Symbol::Line {
                        diagonal: true,
                        color: cur_color,
                    };
                }
                KeyCode::Char('c' | 'C') => {
                    grid.contents[cx][cy].symbol = Symbol::Pips {
                        count: 1,
                        color: cur_color,
                    };
                }
                KeyCode::Char('f' | 'F') => {
                    grid.contents[cx][cy].symbol = Symbol::Petals { count: 0 }
                }
                KeyCode::Char('o' | 'O') => {
                    grid.contents[cx][cy].symbol = Symbol::Lozange { color: cur_color }
                }
                KeyCode::Char('[') => {
                    cur_color = cur_color.prev();
                }
                KeyCode::Char(']') => {
                    cur_color = cur_color.next();
                }
                KeyCode::Char('+' | '=') => match &mut grid.contents[cx][cy].symbol {
                    Symbol::Pips { count, .. } => {
                        if *count != -1 {
                            *count += 1;
                        } else {
                            *count = 1;
                        }
                    }
                    Symbol::Petals { count, .. } => {
                        if *count < 4 {
                            *count += 1;
                        }
                    }
                    _ => (),
                },
                KeyCode::Char('-') => match &mut grid.contents[cx][cy].symbol {
                    Symbol::Pips { count, .. } => {
                        if *count != 1 {
                            *count -= 1;
                        } else {
                            *count = -1;
                        }
                    }
                    Symbol::Petals { count, .. } => {
                        if *count > 0 {
                            *count -= 1;
                        }
                    }
                    _ => (),
                },
                KeyCode::Char('x') => {
                    let out = grid.to_string();
                    if let Some(out_path) = args.out_file.as_ref() {
                        std::fs::write(out_path, out)?;
                    } else {
                        let now = chrono::Utc::now();
                        let fname = format!("{}.tai", now.format("%Y%m%dT%H%M%S"));
                        std::fs::write(fname, out)?;
                    }
                }
                _ => (),
            }
        }
    }
    Ok(())
}
