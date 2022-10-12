use std::collections::HashSet;

use hyphenation::{Load, Hyphenator};
use panel::COLORS;
use tui::{
    backend::Backend,
    layout::{ Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::{Gridlike, panel::{self, PanelError}, line_splitter};

use self::grid::GridRenderer;
mod grid;

pub fn get_errors<'a, G: Gridlike<'a>>(grid: &'a G) -> Vec<PanelError> {
    grid
    .iter()
    .filter_map(|(x, y, panel)| panel.satisfied(x, y, grid).err())    
    .collect()
}

fn render_help<B: Backend>(f: &mut Frame<B>, area: Rect, state_stack_size: usize) {
    let block = Block::default().borders(Borders::ALL).title("Help");
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    let area = inner_area;

    let mut help = vec![];
    
    help.push(Spans(vec![Span::raw("Arrows: move\n")]));
    help.push(Spans(vec![Span::raw("Space: toggle light\n")]));
    help.push(Spans(vec![Span::raw("Shift-space: Lock cell\n")]));
    help.push(Spans(vec![Span::raw("Ctrl-Space: Mark as unlit\n")]));
    help.push(Spans(vec![Span::raw("C: Pips\n")]));
    help.push(Spans(vec![Span::raw("L: lines (l for -, L for /)\n")]));
    help.push(Spans(vec![Span::raw("F: Petals (flower)\n")]));
    help.push(Spans(vec![Span::raw("O: Lozange\n")]));
    help.push(Spans(vec![Span::raw("Tab: Rotate\n")]));
    help.push(Spans(vec![Span::raw("R: Reset\n")]));
    help.push(Spans(vec![Span::raw("Enter: Tag panel\n")]));
    help.push(Spans(vec![Span::raw("0..9: set panel count (where applicable)\n")]));
    help.push(Spans(vec![Span::raw("X: Export\n")]));
    
    help.push(Spans(vec![]));
    help.push(Spans(vec![Span::raw(format!("Saved states: {state_stack_size}"))]));
    let para = Paragraph::new(help).wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn render_errors<'a, B: Backend, H: Hyphenator<'a, Opportunity = usize>>(f: &mut Frame<B>, area: Rect, errors: &[PanelError], hyphenator: &'a H) {
    let block = Block::default().borders(Borders::ALL).title("Errors");
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    let area = inner_area;

    let mut list_items = vec![];
    for err in errors {
        let text = err.to_string();
        let lines = line_splitter::break_lines(&text, area.width as i32 - 3, hyphenator);
        let mut lines: Vec<Spans<'static>> = lines
            .into_iter()
            .map(|line| Spans(vec![Span::raw(line)]))
            .collect();
        for (i, line) in lines.iter_mut().enumerate() {
            if i == 0 {
                line.0.insert(0, Span::raw("* ".to_string()));
            } else {
                line.0.insert(0, Span::raw("  ".to_string()));
            }
        }
        list_items.push(ListItem::new(lines));
    }
    let list = List::new(list_items);
    f.render_widget(list, area);
}

struct ColorPicker { color: panel::Color }
impl tui::widgets::Widget for ColorPicker {
    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
        let block = Block::default().borders(Borders::ALL).title("Colour");
        let inner_area = block.inner(area);
        block.render(area, buf);
        let area = inner_area;
        let mut color_box = area.clone();
        let color_width = area.width / COLORS.len() as u16;
        let padding = (area.width - (COLORS.len() as u16) * color_width) / 2;
        
        color_box.width = color_width;
        color_box.x += padding;

        for color in COLORS {
            buf.set_style(color_box, Style::default().bg(color.to_tui()));
            if color == self.color {
                buf.set_string(color_box.left() + color_width / 2, color_box.top(), "*", Style::default().fg(color.complement().to_tui()));
            }
            color_box.x += color_width;
        }
    }
}

pub fn ui<'a, G: Gridlike<'a>, B: Backend>(
    f: &mut Frame<B>,
    grid: &'a mut G,
    cx: usize,
    cy: usize,
    cur_color: panel::Color,
    state_stack_size: usize,
    tagged: HashSet<(usize, usize)>
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
        .split(f.size());

    let block = Block::default()
        .title("Grid")
        .borders(Borders::all())
        .border_style(Style::default().fg(Color::White));

    let errors = get_errors(grid);
    let error_locs: HashSet<_> = errors.iter().map(|e| e.get_pos()).collect();

    f.render_stateful_widget(GridRenderer::new(&*grid), block.inner(chunks[0]), &mut (cx, cy, tagged, error_locs));
    f.render_widget(block, chunks[0]);
        
    let en_us = hyphenation::Standard::from_embedded(hyphenation::Language::EnglishUS).unwrap();
    let subchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(16), Constraint::Min(30)].as_ref())
        .split(chunks[1]);
    f.render_widget(ColorPicker { color: cur_color }, subchunks[0]);
    render_help(f, subchunks[1], state_stack_size);
    render_errors(f, subchunks[2], &errors, &en_us);

}

