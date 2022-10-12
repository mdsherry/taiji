
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
    pub fn complement(self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::Yellow => Color::Purple,
            Color::Purple => Color::Yellow,
            Color::White => Color::Black,
            Color::Blue => Color::Purple,
            Color::Green => Color::White,
        }
    }
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
