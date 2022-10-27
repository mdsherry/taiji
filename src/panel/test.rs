use crate::grid::{Grid, Gridlike};

use super::{Color, Symbol};

fn check_satisfiable_petals(grid: &mut Grid, config: u8, valid_at: [bool; 3]) {
    const POSITIONS: [(usize, usize); 4] = [(1, 0), (0, 1), (2, 1), (1, 2)];
    for i in 0..4 {
        grid.set_lit_at(POSITIONS[i].0, POSITIONS[i].1, config & (1 << i) != 0);
    }
    grid.set_lit_at(1, 1, true);
    let panel = grid.panel_at(1, 1);
    assert_eq!(panel.satisfiable(1, 1, 0, 1, grid), true);
    assert_eq!(panel.satisfiable(1, 1, 1, 1, grid), valid_at[0]);
    assert_eq!(panel.satisfiable(1, 1, 2, 1, grid), valid_at[1]);
    assert_eq!(panel.satisfiable(1, 1, 0, 2, grid), valid_at[1]);
    assert_eq!(panel.satisfiable(1, 1, 1, 2, grid), valid_at[2]);
    assert_eq!(panel.satisfiable(1, 1, 2, 2, grid), valid_at[2]);

    // Invert everything
    for i in 0..4 {
        grid.set_lit_at(POSITIONS[i].0, POSITIONS[i].1, config & (1 << i) == 0);
    }
    grid.set_lit_at(1, 1, false);
    let panel = grid.panel_at(1, 1);
    assert_eq!(panel.satisfiable(1, 1, 0, 1, grid), true);
    assert_eq!(panel.satisfiable(1, 1, 1, 1, grid), valid_at[0]);
    assert_eq!(panel.satisfiable(1, 1, 2, 1, grid), valid_at[1]);
    assert_eq!(panel.satisfiable(1, 1, 0, 2, grid), valid_at[1]);
    assert_eq!(panel.satisfiable(1, 1, 1, 2, grid), valid_at[2]);
    assert_eq!(panel.satisfiable(1, 1, 2, 2, grid), valid_at[2]);
}

#[test]
fn test_petal_0_satisfiable() {
    let mut grid = Grid::new(3, 3);
    grid.set_symbol_at(1, 1, Symbol::Petals { count: 0 });

    check_satisfiable_petals(&mut grid, 0b0000, [true, true, true]);
    // Any case where either of the first two petals are set will always be false for any position where we don't short-circuit early (because we might not have set this panel's colour yet)
    check_satisfiable_petals(&mut grid, 0b0001, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b0011, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b0101, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b0111, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1001, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1011, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1101, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1111, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b0010, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b0110, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1010, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1110, [false, false, false]);
    // If the third position is set, we're only satisfiable in the first check
    check_satisfiable_petals(&mut grid, 0b0100, [true, false, false]);
    check_satisfiable_petals(&mut grid, 0b1100, [true, false, false]);
    // Fourth position means we pass the first two checks
    check_satisfiable_petals(&mut grid, 0b1000, [true, true, false]);
}

#[test]
fn test_petal_1_satisfiable() {
    let mut grid = Grid::new(3, 3);
    grid.set_symbol_at(1, 1, Symbol::Petals { count: 1 });

    // Empty grid is ultimately unsatisfiable, but we don't know that until we've checked them all
    check_satisfiable_petals(&mut grid, 0b0000, [true, true, false]);
    // Any case where we only have one set is always satisfiable
    check_satisfiable_petals(&mut grid, 0b0001, [true, true, true]);
    check_satisfiable_petals(&mut grid, 0b0010, [true, true, true]);
    check_satisfiable_petals(&mut grid, 0b0100, [true, true, true]);
    check_satisfiable_petals(&mut grid, 0b1000, [true, true, true]);
    // We're always false if the first two are set
    check_satisfiable_petals(&mut grid, 0b0011, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b0111, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1011, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1111, [false, false, false]);
    // Otherwise we're false once we've seen too many
    check_satisfiable_petals(&mut grid, 0b0101, [true, false, false]);
    check_satisfiable_petals(&mut grid, 0b1001, [true, true, false]);
    check_satisfiable_petals(&mut grid, 0b1010, [true, true, false]);
    check_satisfiable_petals(&mut grid, 0b1101, [true, false, false]);
    check_satisfiable_petals(&mut grid, 0b0110, [true, false, false]);
    check_satisfiable_petals(&mut grid, 0b1110, [true, false, false]);
    check_satisfiable_petals(&mut grid, 0b1100, [true, true, false]);
}

#[test]
fn test_petal_2_satisfiable() {
    let mut grid = Grid::new(3, 3);
    grid.set_symbol_at(1, 1, Symbol::Petals { count: 2 });

    check_satisfiable_petals(&mut grid, 0b0000, [true, false, false]);
    // Any case where we only have two set is always satisfiable
    check_satisfiable_petals(&mut grid, 0b0011, [true, true, true]);
    check_satisfiable_petals(&mut grid, 0b0101, [true, true, true]);
    check_satisfiable_petals(&mut grid, 0b1001, [true, true, true]);
    check_satisfiable_petals(&mut grid, 0b1010, [true, true, true]);
    check_satisfiable_petals(&mut grid, 0b0110, [true, true, true]);
    check_satisfiable_petals(&mut grid, 0b1100, [true, true, true]);

    // One set fails at the end
    check_satisfiable_petals(&mut grid, 0b0001, [true, true, false]);
    check_satisfiable_petals(&mut grid, 0b0010, [true, true, false]);
    check_satisfiable_petals(&mut grid, 0b0100, [true, true, false]);
    check_satisfiable_petals(&mut grid, 0b1000, [true, false, false]);

    check_satisfiable_petals(&mut grid, 0b0111, [true, false, false]);
    check_satisfiable_petals(&mut grid, 0b1011, [true, true, false]);
    check_satisfiable_petals(&mut grid, 0b1111, [true, false, false]);
    check_satisfiable_petals(&mut grid, 0b1101, [true, true, false]);
    check_satisfiable_petals(&mut grid, 0b1110, [true, true, false]);
}

#[test]
fn test_petal_3_satisfiable() {
    let mut grid = Grid::new(3, 3);
    grid.set_symbol_at(1, 1, Symbol::Petals { count: 3 });

    // If neither of the first two are checked, we can never reach the goal
    check_satisfiable_petals(&mut grid, 0b0000, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1100, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b0100, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1000, [false, false, false]);

    // Three set is always satisfiable
    check_satisfiable_petals(&mut grid, 0b0111, [true, true, true]);
    check_satisfiable_petals(&mut grid, 0b1011, [true, true, true]);
    check_satisfiable_petals(&mut grid, 0b1101, [true, true, true]);
    check_satisfiable_petals(&mut grid, 0b1110, [true, true, true]);

    // If only one of the first two is set, it fails in 2nd position if 3rd is unset
    check_satisfiable_petals(&mut grid, 0b1001, [true, false, false]);
    check_satisfiable_petals(&mut grid, 0b1010, [true, false, false]);
    check_satisfiable_petals(&mut grid, 0b0001, [true, false, false]);
    check_satisfiable_petals(&mut grid, 0b0010, [true, false, false]);

    // Two set fails at the end
    check_satisfiable_petals(&mut grid, 0b0011, [true, true, false]);
    check_satisfiable_petals(&mut grid, 0b0101, [true, true, false]);
    check_satisfiable_petals(&mut grid, 0b0110, [true, true, false]);

    check_satisfiable_petals(&mut grid, 0b1111, [true, true, false]);
}

#[test]
fn test_petal_4_satisfiable() {
    let mut grid = Grid::new(3, 3);
    grid.set_symbol_at(1, 1, Symbol::Petals { count: 4 });

    // The only valid case
    check_satisfiable_petals(&mut grid, 0b1111, [true, true, true]);

    // Otherwise, we fail once we see the first unset
    check_satisfiable_petals(&mut grid, 0b0000, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1100, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b0100, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1000, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1101, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1110, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1001, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b1010, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b0001, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b0010, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b0101, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b0110, [false, false, false]);
    check_satisfiable_petals(&mut grid, 0b0111, [true, true, false]);
    check_satisfiable_petals(&mut grid, 0b1011, [true, false, false]);
    check_satisfiable_petals(&mut grid, 0b0011, [true, false, false]);
}

fn with_symbol_at<F>(grid: &mut Grid, x: usize, y: usize, symbol: Symbol, f: F)
where
    F: FnOnce(&mut Grid),
{
    let old = grid.symbol_at(x, y);
    grid.set_symbol_at(x, y, symbol);
    f(grid);
    grid.set_symbol_at(x, y, old.unwrap_or(Symbol::Plain));
}

#[test]
fn test_line() {
    // We only check that a line, when translated to align with any matching line symbols, doesn't go out-of-bounds
    // We could also check that we don't duplicate shapes, but it's hard to know when all relevant shapes can no longer grow.
    let mut grid = Grid::new(6, 6);
    // Make a pattern like
    //  #
    // #-#
    //   #
    grid.set_symbol_at(
        1,
        1,
        Symbol::Line {
            diagonal: false,
            color: Color::Blue,
        },
    );
    grid.set_lit_at(1, 0, true);
    grid.set_lit_at(0, 1, true);
    grid.set_lit_at(1, 1, true);
    grid.set_lit_at(2, 1, true);
    grid.set_lit_at(2, 2, true);

    let panel = grid.panel_at(1, 1);
    // No other lines; it's fine
    assert!(panel.satisfiable(1, 1, 1, 1, &grid));
    // It's the wrong color, so it's fine
    with_symbol_at(
        &mut grid,
        0,
        4,
        Symbol::Line {
            diagonal: false,
            color: Color::Green,
        },
        |grid| {
            assert!(panel.satisfiable(1, 1, 1, 1, grid));
        },
    );

    with_symbol_at(
        &mut grid,
        0,
        4,
        Symbol::Line {
            diagonal: false,
            color: Color::Blue,
        },
        |grid| {
            assert!(!panel.satisfiable(1, 1, 1, 1, grid));
        },
    );

    // Check top row
    with_symbol_at(
        &mut grid,
        4,
        0,
        Symbol::Line {
            diagonal: false,
            color: Color::Blue,
        },
        |grid| {
            assert!(!panel.satisfiable(1, 1, 1, 1, grid));
        },
    );

    // Right-most column is fine until we start looking further
    with_symbol_at(
        &mut grid,
        5,
        2,
        Symbol::Line {
            diagonal: false,
            color: Color::Blue,
        },
        |grid| {
            assert!(panel.satisfiable(1, 1, 1, 1, grid));
            assert!(!panel.satisfiable(1, 1, 2, 1, grid));
        },
    );

    // Bottom row
    with_symbol_at(
        &mut grid,
        2,
        5,
        Symbol::Line {
            diagonal: false,
            color: Color::Blue,
        },
        |grid| {
            assert!(panel.satisfiable(1, 1, 1, 1, grid));
            assert!(panel.satisfiable(1, 1, 2, 1, grid));
            assert!(panel.satisfiable(1, 1, 1, 2, grid));
            assert!(!panel.satisfiable(1, 1, 2, 2, grid));
        },
    );

    // Middle is fine all the time
    with_symbol_at(
        &mut grid,
        3,
        3,
        Symbol::Line {
            diagonal: false,
            color: Color::Blue,
        },
        |grid| {
            assert!(panel.satisfiable(1, 1, 1, 1, grid));
            assert!(panel.satisfiable(1, 1, 2, 1, grid));
            assert!(panel.satisfiable(1, 1, 1, 2, grid));
            assert!(panel.satisfiable(1, 1, 2, 2, grid));
        },
    );
}

#[test]
fn test_diagonal_line() {
    // We only check that a line, when translated to align with any matching line symbols, doesn't go out-of-bounds in at least one orientation
    // We could also check that we don't duplicate shapes, but it's hard to know when all relevant shapes can no longer grow.
    let mut grid = Grid::new(6, 6);
    // Make a pattern like
    //  #
    // #-#
    //   #
    // which can fit anywhere for up to (1, 1), anywhere but corners for up to (1, 2), and then anywhere but corners or edges
    grid.set_symbol_at(
        1,
        1,
        Symbol::Line {
            diagonal: true,
            color: Color::Blue,
        },
    );
    grid.set_lit_at(1, 0, true);
    grid.set_lit_at(0, 1, true);
    grid.set_lit_at(1, 1, true);
    grid.set_lit_at(2, 1, true);
    grid.set_lit_at(2, 2, true);

    let panel = grid.panel_at(1, 1);
    // No other lines; it's fine
    assert!(panel.satisfiable(1, 1, 1, 1, &grid));
    // It's the wrong color, so it's fine
    with_symbol_at(
        &mut grid,
        0,
        4,
        Symbol::Line {
            diagonal: false,
            color: Color::Green,
        },
        |grid| {
            assert!(panel.satisfiable(1, 1, 1, 1, grid));
        },
    );

    with_symbol_at(
        &mut grid,
        0,
        4,
        Symbol::Line {
            diagonal: false,
            color: Color::Blue,
        },
        |grid| {
            assert!(panel.satisfiable(1, 1, 1, 1, grid));
        },
    );

    // Check top row
    with_symbol_at(
        &mut grid,
        4,
        0,
        Symbol::Line {
            diagonal: false,
            color: Color::Blue,
        },
        |grid| {
            assert!(panel.satisfiable(1, 1, 1, 1, grid));
            assert!(panel.satisfiable(1, 1, 1, 2, grid));
            assert!(panel.satisfiable(1, 1, 2, 1, grid));
            assert!(!panel.satisfiable(1, 1, 2, 2, grid));
        },
    );

    // Right-most column is fine until we start looking further
    with_symbol_at(
        &mut grid,
        5,
        2,
        Symbol::Line {
            diagonal: false,
            color: Color::Blue,
        },
        |grid| {
            assert!(panel.satisfiable(1, 1, 1, 1, grid));
            assert!(panel.satisfiable(1, 1, 2, 1, grid));
            assert!(!panel.satisfiable(1, 1, 2, 2, grid));
        },
    );

    // Bottom row
    with_symbol_at(
        &mut grid,
        2,
        5,
        Symbol::Line {
            diagonal: false,
            color: Color::Blue,
        },
        |grid| {
            assert!(panel.satisfiable(1, 1, 1, 1, grid));
            assert!(panel.satisfiable(1, 1, 2, 1, grid));
            assert!(panel.satisfiable(1, 1, 1, 2, grid));
            assert!(!panel.satisfiable(1, 1, 2, 2, grid));
        },
    );

    // Middle is fine all the time
    with_symbol_at(
        &mut grid,
        3,
        3,
        Symbol::Line {
            diagonal: false,
            color: Color::Blue,
        },
        |grid| {
            assert!(panel.satisfiable(1, 1, 1, 1, grid));
            assert!(panel.satisfiable(1, 1, 2, 1, grid));
            assert!(panel.satisfiable(1, 1, 1, 2, grid));
            assert!(panel.satisfiable(1, 1, 2, 2, grid));
        },
    );

    // Corner bcomes un-fine after (2, 1)
    with_symbol_at(
        &mut grid,
        5,
        5,
        Symbol::Line {
            diagonal: false,
            color: Color::Blue,
        },
        |grid| {
            assert!(panel.satisfiable(1, 1, 1, 1, grid));
            assert!(!panel.satisfiable(1, 1, 2, 1, grid));
            assert!(!panel.satisfiable(1, 1, 1, 2, grid));
            assert!(!panel.satisfiable(1, 1, 2, 2, grid));
        },
    );
}

#[test]
fn test_pips() {
    // Pips are unsatisfiable if
    //   a) they're sharing space with another pip of a different colour
    //   b) the neighbourhood is too large to ever pay off (unless pips all add up to 0)
    let mut grid = Grid::new(6, 6);
    grid.set_symbol_at(
        0,
        0,
        Symbol::Pips {
            count: 3,
            color: Color::Blue,
        },
    );
    let panel = grid.panel_at(0, 0);
    // Not bothering to colour: we'll assume our neighbourhood is unlit
    assert!(panel.satisfiable(0, 0, 0, 0, &grid));
    assert!(panel.satisfiable(0, 0, 1, 0, &grid));
    assert!(panel.satisfiable(0, 0, 2, 0, &grid));
    assert!(!panel.satisfiable(0, 0, 3, 0, &grid));

    with_symbol_at(
        &mut grid,
        1,
        0,
        Symbol::Pips {
            count: 2,
            color: Color::Green,
        },
        |grid| {
            assert!(panel.satisfiable(0, 0, 0, 0, grid));
            // Now we can see the green pip
            assert!(!panel.satisfiable(0, 0, 1, 0, grid));
        },
    );

    with_symbol_at(
        &mut grid,
        2,
        0,
        Symbol::Pips {
            count: 4,
            color: Color::Blue,
        },
        |grid| {
            // This is fine now, because we've got another pip in our neighbourhood
            assert!(panel.satisfiable(0, 0, 3, 0, grid));
            // But this is too much
            assert!(!panel.satisfiable(0, 0, 3, 1, grid));
        },
    );

    with_symbol_at(
        &mut grid,
        2,
        2,
        Symbol::Pips {
            count: 4,
            color: Color::Blue,
        },
        |grid| {
            // This is also fine, since we could (in theory) meet up with this other pip to get bigger
            assert!(panel.satisfiable(0, 0, 3, 0, grid));
            // But this is too much
            assert!(!panel.satisfiable(0, 0, 3, 1, grid));
        },
    );

    with_symbol_at(
        &mut grid,
        2,
        2,
        Symbol::Pips {
            count: -3,
            color: Color::Blue,
        },
        |grid| {
            // This is also fine, since if we meet up with the -3, we'll balance out to 0
            assert!(panel.satisfiable(0, 0, 3, 0, grid));
            // This is fine too!
            assert!(panel.satisfiable(0, 0, 3, 1, grid));
        },
    );

    with_symbol_at(
        &mut grid,
        2,
        2,
        Symbol::Pips {
            count: -2,
            color: Color::Blue,
        },
        |grid| {
            // -2 isn't enough: we need a total of -3
            assert!(!panel.satisfiable(0, 0, 3, 0, grid));
            with_symbol_at(
                grid,
                2,
                3,
                Symbol::Pips {
                    count: -5,
                    color: Color::Blue,
                },
                |grid| {
                    // This is fine again, even though it's TOO MUCH: we're not solving packing problems at this time
                    assert!(panel.satisfiable(0, 0, 3, 0, grid));
                },
            );
        },
    );
}

#[test]
fn test_lozange() {
    let mut grid = Grid::new(6, 6);
    grid.set_symbol_at(
        1,
        1,
        Symbol::Lozange {
            color: Color::Purple,
        },
    );
    let panel = grid.panel_at(1, 1);
    // A single lozange is still "satisfiable": even though a single lozange on the board isn't actually satisfiable, that's a function of the board, not our solution
    assert!(panel.satisfiable(1, 1, 5, 5, &grid));
    // A second lozange is also fine
    grid.set_symbol_at(
        3,
        1,
        Symbol::Lozange {
            color: Color::Purple,
        },
    );
    assert!(panel.satisfiable(1, 1, 5, 5, &grid));

    with_symbol_at(
        &mut grid,
        2,
        2,
        Symbol::Lozange {
            color: Color::Green,
        },
        |grid| {
            // Another third lozange of a different colour is fine
            assert!(panel.satisfiable(1, 1, 5, 5, grid));
        },
    );

    with_symbol_at(
        &mut grid,
        2,
        2,
        Symbol::Lozange {
            color: Color::Purple,
        },
        |grid| {
            // Another third lozange of the same is not
            assert!(panel.satisfiable(1, 1, 1, 2, grid));
            assert!(!panel.satisfiable(1, 1, 2, 2, grid));
        },
    );

    with_symbol_at(
        &mut grid,
        2,
        2,
        Symbol::Pips {
            count: 3,
            color: Color::Purple,
        },
        |grid| {
            // Pips count
            assert!(panel.satisfiable(1, 1, 1, 2, grid));
            assert!(!panel.satisfiable(1, 1, 2, 2, grid));
        },
    );
    with_symbol_at(
        &mut grid,
        2,
        2,
        Symbol::Line {
            diagonal: false,
            color: Color::Purple,
        },
        |grid| {
            // As do lines
            assert!(panel.satisfiable(1, 1, 1, 2, grid));
            assert!(!panel.satisfiable(1, 1, 2, 2, grid));
        },
    );

    with_symbol_at(&mut grid, 2, 2, Symbol::Petals { count: 0 }, |grid| {
        // Empty petals count as purple
        assert!(panel.satisfiable(1, 1, 1, 2, grid));
        assert!(!panel.satisfiable(1, 1, 2, 2, grid));
    });
    with_symbol_at(&mut grid, 2, 2, Symbol::Petals { count: 3 }, |grid| {
        // So do non-full ones
        assert!(panel.satisfiable(1, 1, 1, 2, grid));
        assert!(!panel.satisfiable(1, 1, 2, 2, grid));
    });
    with_symbol_at(&mut grid, 2, 2, Symbol::Petals { count: 4 }, |grid| {
        // But full ones count as yellow!
        assert!(panel.satisfiable(1, 1, 1, 2, grid));
        assert!(panel.satisfiable(1, 1, 2, 2, grid));
        with_symbol_at(
            grid,
            1,
            1,
            Symbol::Lozange {
                color: Color::Yellow,
            },
            |grid| {
                let panel = grid.panel_at(1, 1);
                assert!(panel.satisfiable(1, 1, 1, 2, grid));
                assert!(panel.satisfiable(1, 1, 2, 2, grid));
                with_symbol_at(
                    grid,
                    3,
                    1,
                    Symbol::Lozange {
                        color: Color::Yellow,
                    },
                    |grid| {
                        assert!(panel.satisfiable(1, 1, 1, 2, grid));
                        assert!(!panel.satisfiable(1, 1, 2, 2, grid));
                    },
                );
            },
        );
    });
    with_symbol_at(&mut grid, 2, 2, Symbol::Petals { count: 2 }, |grid| {
        // As do partial flowers
        with_symbol_at(
            grid,
            1,
            1,
            Symbol::Lozange {
                color: Color::Yellow,
            },
            |grid| {
                let panel = grid.panel_at(1, 1);
                assert!(panel.satisfiable(1, 1, 1, 2, grid));
                assert!(panel.satisfiable(1, 1, 2, 2, grid));
                with_symbol_at(
                    grid,
                    3,
                    1,
                    Symbol::Lozange {
                        color: Color::Yellow,
                    },
                    |grid| {
                        assert!(panel.satisfiable(1, 1, 1, 2, grid));
                        assert!(!panel.satisfiable(1, 1, 2, 2, grid));
                    },
                );
            },
        );
    });
}
