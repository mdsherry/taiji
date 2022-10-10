use hyphenation::Hyphenator;
use paragraph_breaker::{Item, standard_fit};
use thiserror::Error;

pub fn break_lines<'a, H>(
    text: &str,
    width: i32,
    hyphenator: &'a H,
) -> Result<Vec<String>, LinebreakError>
where
    H: Hyphenator<'a, Opportunity = usize>,
{
    let mut rv = vec![];
    
    // How wide is a space?
    let space_width = 1;
    let hyphen_width = 1;
    let mut fragments = vec![];
    fragments.push(Item::Glue { width: 0, stretch: 10, shrink: 10 });
    for word in text.split_whitespace() {
        let hyph = hyphenator.hyphenate(word);

        let mut last = 0;
        for &brk in &hyph.breaks {
            let subword = &word[last..brk];
            
            fragments.push(Item::Box {
                width: subword.chars().count() as i32,
                data: subword,
            });
           
            fragments.push(Item::Penalty {
                width: hyphen_width,
                penalty: 10000,
                flagged: false,
            });
            last = brk;
        }
        let subword = &word[last..];
        
        fragments.push(Item::Box {
            width: subword.chars().count() as i32,
            data: subword,
        });
        
        fragments.push(Item::Glue {
            width: space_width,
            stretch: 0,
            shrink: 0,
        });
    }
    // dbg!(&fragments);
    // Now for the simple bit: just split the fragments into buckets as evenly sized as possible
    let fit = standard_fit(&fragments, &[width; 100], 1.);
    
    let mut last = 0;
    for breakpoint in &fit {
        let mut line = build_line(&fragments, last, breakpoint.index);
        last = breakpoint.index;
        match fragments[breakpoint.index] {
            Item::Box { .. } => (), // Probably impossible?
            Item::Glue { .. } => last += 1,
            Item::Penalty { width, .. } => {
                last += 1;
                if width > 0 {
                    line.push('-');
                }
            }
        }
        
        rv.push(line);
    }
    let line = build_line(&fragments, last, fragments.len());
    rv.push(line);
    
    Ok(rv)
}

fn build_line(fragments: &[Item<&str>], start: usize, end: usize) -> String {
    let mut line = String::new();
    for (i, fragment) in fragments[start..end].iter().enumerate() {
        match fragment {
            Item::Box { data, .. } => line.push_str(data),
            Item::Glue { .. } => if i > 0 { line.push(' ') },
            Item::Penalty { .. } => (),
        }
    }

    if matches!(fragments[end - 1], Item::Penalty { .. }) {
        eprintln!("Penalty at the end!");
        line.push('-');
    }
    line
}


#[derive(Error, Debug, Clone)]
pub enum LinebreakError {
    #[error("Line {text} requires {required} units of space, but only {have} available")]
    LineDoesntFit {
        text: String,
        required: i32, have: i32
    },
    #[error("No line widths provided")]
    NoLines,
    #[error("Too many lines: only {widths} widths available")]
    TooManyLines { widths: usize }
}
