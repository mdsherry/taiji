use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use std::{
    collections::HashSet,
    io,
};
use tui::{backend::Backend, Terminal};

use crate::{
    grid::{Grid2, Gridlike},
    panel::{self, Symbol},
    render::ui,
    Args,
};

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, args: Args) -> io::Result<()> {
    let mut grid = if let Some(in_path) = args.in_file {
        let s = std::fs::read_to_string(in_path)?;
        s.parse().unwrap()
    } else {
        Grid2::new(args.width, args.height)
    };
    
    let mut cx = 0;
    let mut cy = 0;
    let mut tagged = HashSet::new();
    let mut cur_color = panel::Color::Yellow;
    let mut state_stack = vec![];
    loop {
        terminal.draw(|f| {
            ui(
                f,
                &mut grid,
                cx,
                cy,
                cur_color,
                state_stack.len(),
                tagged.clone(),
            )
        })?;

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
                KeyCode::Char(n @ '1'..='9') => {
                    let n = (n as u8 - b'1' + 1) as usize;
                    grid.symbol_at_mut(cx, cy).map(|s| s.set_count(n as i8));
                }
                KeyCode::Tab => {
                    grid.rotate();
                }
                KeyCode::Up => {
                    if cy > 0 {
                        cy -= 1;
                    }
                }
                KeyCode::Down => {
                    if cy + 1 < grid.height() {
                        cy += 1;
                    }
                }
                KeyCode::Left => {
                    if cx > 0 {
                        cx -= 1;
                    }
                }
                KeyCode::Right => {
                    if cx + 1 < grid.width() {
                        cx += 1;
                    }
                }
                KeyCode::Char(' ') => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        grid.toggle_fixed_at(cx, cy);
                    } else if key.modifiers.contains(KeyModifiers::CONTROL) {
                        if !grid.fixed_at(cx, cy) {
                            grid.toggle_colit_at(cx, cy);
                            grid.set_lit_at(cx, cy, false);
                        }
                    } else {
                        if !grid.fixed_at(cx, cy) {
                            grid.toggle_lit_at(cx, cy);
                            grid.set_colit_at(cx, cy, false);
                        }
                    }
                }
                KeyCode::Char('r') => {
                    grid.reset();
                }
                KeyCode::Char('.') => {
                    grid.set_symbol_at(cx, cy, Symbol::Plain);
                }
                KeyCode::Char('l') => {
                    grid.set_symbol_at(
                        cx,
                        cy,
                        Symbol::Line {
                            diagonal: false,
                            color: cur_color,
                        },
                    );
                }
                KeyCode::Char('L') => {
                    grid.set_symbol_at(
                        cx,
                        cy,
                        Symbol::Line {
                            diagonal: true,
                            color: cur_color,
                        },
                    );
                }
                KeyCode::Char('c' | 'C') => {
                    grid.set_symbol_at(
                        cx,
                        cy,
                        Symbol::Pips {
                            count: 1,
                            color: cur_color,
                        },
                    );
                }
                KeyCode::Char('f' | 'F') => {
                    grid.set_symbol_at(cx, cy, Symbol::Petals { count: 0 });
                }
                KeyCode::Char('o' | 'O') => {
                    grid.set_symbol_at(cx, cy, Symbol::Lozange { color: cur_color });
                }
                KeyCode::Char('[') => {
                    cur_color = cur_color.prev();
                }
                KeyCode::Char(']') => {
                    cur_color = cur_color.next();
                }
                KeyCode::Char('+' | '=') => {
                    grid.symbol_at_mut(cx, cy).map(|s| s.incr_count());
                }
                KeyCode::Char('-') => {
                    grid.symbol_at_mut(cx, cy).map(|s| s.decr_count());
                }
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
