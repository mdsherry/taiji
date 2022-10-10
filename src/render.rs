use std::collections::HashSet;

use hyphenation::{Load, Hyphenator};
use tui::{
    backend::Backend,
    layout::{ Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::{Grid, panel::{self, PanelError}, line_splitter};
mod grid;

pub fn get_errors(grid: &Grid) -> Vec<PanelError> {
    grid
    .contents
    .iter()
    .enumerate()
    .flat_map(|(x, col)| {
        col.iter()
            .enumerate()
            .filter_map(move |(y, panel)| panel.satisfied(x, y, grid).err())
    })
    .collect()
}

fn render_help<B: Backend>(f: &mut Frame<B>, area: Rect, color: panel::Color, state_stack_size: usize) {
    let block = Block::default().borders(Borders::ALL).title("Help");
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    let area = inner_area;

    let mut help = vec![];
    help.push(Spans(vec![Span::styled(
        format!("{:?}\n", color),
        Style::default().fg(color.to_tui()).bg(Color::DarkGray),
    )]));
    help.push(Spans(vec![Span::raw("Arrows: move\n")]));
    help.push(Spans(vec![Span::raw("Space: toggle light\n")]));
    help.push(Spans(vec![Span::raw("C: Pips\n")]));
    help.push(Spans(vec![Span::raw("L: lines (l for -, L for /)\n")]));
    help.push(Spans(vec![Span::raw("F: Petals (flower)\n")]));
    help.push(Spans(vec![Span::raw("O: Lozange\n")]));
    help.push(Spans(vec![]));
    help.push(Spans(vec![Span::raw(format!("Saved states: {state_stack_size}"))]));
    let para = Paragraph::new(help);
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
        let lines = line_splitter::break_lines(&text, area.width as i32 - 3, hyphenator).unwrap();
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

pub fn ui<B: Backend>(
    f: &mut Frame<B>,
    grid: &mut Grid,
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

    f.render_stateful_widget(&*grid, block.inner(chunks[0]), &mut (cx, cy, tagged, error_locs));
    f.render_widget(block, chunks[0]);
    
    
    let en_us = hyphenation::Standard::from_embedded(hyphenation::Language::EnglishUS).unwrap();
    
    

    let subchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(15), Constraint::Min(30)].as_ref())
        .split(chunks[1]);
    render_help(f, subchunks[0], cur_color, state_stack_size);
    render_errors(f, subchunks[1], &errors, &en_us);

}

