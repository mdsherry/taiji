use crate::{
    grid::{Grid, Gridlike, Neighbourhood, Rotation, ROTATIONS},
    panel::Panel,
};
use indoc::indoc;

static TEST_NEIGHBOURHOOD_S: &str = indoc! {"
    2E
     ##
     O 
    ## \n
"};

static TEST_NEIGHBOURHOOD_L1: &str = indoc! {"
2B
##
O 
"};
static TEST_NEIGHBOURHOOD_L2: &str = indoc! {"
2E
#O
# 
"};

static TEST_NEIGHBOURHOOD_I: &str = indoc! {"
1A
O##
"};

#[test]
fn test_parse() {
    let n: Neighbourhood = TEST_NEIGHBOURHOOD_S.parse().unwrap();
    assert_eq!(1, n.offset_x);
    assert_eq!(4, n.offset_y);
    assert_eq!(5, n.contents.len());
    assert_eq!(TEST_NEIGHBOURHOOD_S, n.to_string());
}

#[test]
fn test_new_around() {
    let g = Grid::new(3, 3);
    let n = Neighbourhood::new_around(1, 1, &g);
    let expected = Neighbourhood {
        offset_x: 1,
        offset_y: 1,
        contents: vec![
            (0, -1, Panel::default()),
            (-1, 0, Panel::default()),
            (0, 0, Panel::default()),
            (1, 0, Panel::default()),
            (0, 1, Panel::default()),
        ],
    };
    assert_eq!(expected, n);

    let g = Grid::new(3, 3);
    let n = Neighbourhood::new_around(0, 0, &g);
    let expected = Neighbourhood {
        offset_x: 0,
        offset_y: 0,
        contents: vec![
            (0, 0, Panel::default()),
            (1, 0, Panel::default()),
            (0, 1, Panel::default()),
        ],
    };
    assert_eq!(expected, n);

    let g = Grid::new(3, 3);
    let n = Neighbourhood::new_around(2, 2, &g);
    let expected = Neighbourhood {
        offset_x: 2,
        offset_y: 2,
        contents: vec![
            (0, -1, Panel::default()),
            (-1, 0, Panel::default()),
            (0, 0, Panel::default()),
        ],
    };
    assert_eq!(expected, n);
}

#[test]
fn test_translate_to() {
    let g = Grid::new(5, 5);
    let n = Neighbourhood::new_around(1, 1, &g);
    let mut n2 = n.clone();
    n2.translate_to(3, 3);
    assert_eq!((3, 3), (n2.offset_x, n2.offset_y));
    assert_eq!(n.contents, n2.contents);
}

#[test]
fn test_inbounds() {
    let mut n: Neighbourhood = TEST_NEIGHBOURHOOD_S.parse().unwrap();
    assert_eq!(true, n.inbounds(3, 6));
    assert_eq!(false, n.inbounds(3, 5));
    assert_eq!(false, n.inbounds(2, 6));
    n.translate_to(0, 4);
    assert_eq!(false, n.inbounds(3, 6));
    n.translate_to(1, 1);
    assert_eq!(true, n.inbounds(3, 3));
    n.translate_to(1, 0);
    assert_eq!(false, n.inbounds(3, 3));
}

#[test]
fn test_rotated_inbounds() {
    let mut n: Neighbourhood = TEST_NEIGHBOURHOOD_L1.parse().unwrap();
    assert_eq!(false, n.inbounds(2, 2));
    assert_eq!(true, n.inbounds_rotated(2, 2));
    n.translate_to(0, 0);
    assert_eq!(true, n.inbounds_rotated(2, 2));

    // A 3x1 should fit everywhere except for the very center
    let mut n: Neighbourhood = TEST_NEIGHBOURHOOD_I.parse().unwrap();
    for x in 0..3 {
        for y in 0..3 {
            n.translate_to(x, y);
            assert_eq!(x != 1 || y != 1, n.inbounds_rotated(3, 3));
        }
    }
}

#[test]
fn test_same_shape() {
    let n1: Neighbourhood = TEST_NEIGHBOURHOOD_S.parse().unwrap();
    let n1_1 = n1.translated_to(2, 7);
    assert!(n1.same_shape(&n1_1));
    let n2: Neighbourhood = TEST_NEIGHBOURHOOD_L1.parse().unwrap();
    let n2_1 = n2.translated_to(2, 7);
    assert!(n2.same_shape(&n2_1));
    let n3: Neighbourhood = TEST_NEIGHBOURHOOD_L2.parse().unwrap();
    let n3_1 = n3.translated_to(2, 7);
    assert!(n3.same_shape(&n3_1));
    let n4: Neighbourhood = TEST_NEIGHBOURHOOD_I.parse().unwrap();
    let n4_1 = n4.translated_to(2, 7);
    assert!(n4.same_shape(&n4_1));

    assert!(!n1.same_shape(&n2));
    assert!(!n1.same_shape(&n3));
    assert!(!n1.same_shape(&n4));
    assert!(!n2.same_shape(&n3));
    assert!(!n2.same_shape(&n4));
    assert!(!n3.same_shape(&n4));
}

fn rotated(n: &Neighbourhood, rot: Rotation) -> Neighbourhood {
    let mut rv = n.clone();
    for (x, y, _) in &mut rv.contents {
        (*x, *y) = rot.rotate((*x, *y));
    }
    rv
}

#[test]
fn test_same_shape_rotated() {
    let n1: Neighbourhood = TEST_NEIGHBOURHOOD_S.parse().unwrap();
    let n1_1 = n1.translated_to(2, 7);
    for rot in ROTATIONS {
        assert!(n1.same_shape_rotated(&rotated(&n1_1, rot)));
    }
    let n2: Neighbourhood = TEST_NEIGHBOURHOOD_L1.parse().unwrap();
    let n2_1 = n2.translated_to(2, 7);
    for rot in ROTATIONS {
        assert!(n2.same_shape_rotated(&rotated(&n2_1, rot)));
    }
    assert!(n2.same_shape_rotated(&n2_1));
    let n3: Neighbourhood = TEST_NEIGHBOURHOOD_L2.parse().unwrap();
    let n3_1 = n3.translated_to(2, 7);
    for rot in ROTATIONS {
        assert!(n3.same_shape_rotated(&rotated(&n3_1, rot)));
    }
    assert!(n3.same_shape_rotated(&n3_1));
    let n4: Neighbourhood = TEST_NEIGHBOURHOOD_I.parse().unwrap();
    let n4_1 = n4.translated_to(2, 7);
    for rot in ROTATIONS {
        assert!(n4.same_shape_rotated(&rotated(&n4_1, rot)));
    }
    assert!(n4.same_shape_rotated(&n4_1));

    assert!(!n1.same_shape_rotated(&n2));
    assert!(!n1.same_shape_rotated(&n3));
    assert!(!n1.same_shape_rotated(&n4));
    assert!(!n2.same_shape_rotated(&n3));
    assert!(!n2.same_shape_rotated(&n4));
    assert!(!n3.same_shape_rotated(&n4));
}

#[test]
fn test_rotated_overlap() {
    let n1: Neighbourhood = TEST_NEIGHBOURHOOD_S.parse().unwrap();
    let n2: Neighbourhood = TEST_NEIGHBOURHOOD_L1.parse().unwrap();
    let n3: Neighbourhood = TEST_NEIGHBOURHOOD_L2.parse().unwrap();
    let n4: Neighbourhood = TEST_NEIGHBOURHOOD_I.parse().unwrap();

    //  11    22
    //  O     O    3O  O44
    // 11          3
    assert_eq!(5, n1.rotated_overlap(&n1, Rotation::D0));
    assert_eq!(3, n1.rotated_overlap(&n2, Rotation::D0));
    assert_eq!(2, n1.rotated_overlap(&n3, Rotation::D0));
    assert_eq!(1, n1.rotated_overlap(&n4, Rotation::D0));
    assert_eq!(3, n2.rotated_overlap(&n2, Rotation::D0));
    assert_eq!(1, n2.rotated_overlap(&n3, Rotation::D0));
    assert_eq!(1, n2.rotated_overlap(&n4, Rotation::D0));
    assert_eq!(3, n3.rotated_overlap(&n3, Rotation::D0));
    assert_eq!(1, n3.rotated_overlap(&n4, Rotation::D0));
    assert_eq!(3, n4.rotated_overlap(&n4, Rotation::D0));

    // 1          33
    // 1O1  O2     O  O
    //   1   2        4
    //                4
    assert_eq!(1, n1.rotated_overlap(&n1, Rotation::D90));
    assert_eq!(1, n1.rotated_overlap(&n2, Rotation::D90));
    assert_eq!(2, n1.rotated_overlap(&n3, Rotation::D90));
    assert_eq!(2, n1.rotated_overlap(&n4, Rotation::D90));
    assert_eq!(1, n2.rotated_overlap(&n2, Rotation::D90));
    assert_eq!(1, n2.rotated_overlap(&n3, Rotation::D90));
    assert_eq!(2, n2.rotated_overlap(&n4, Rotation::D90));
    assert_eq!(1, n3.rotated_overlap(&n3, Rotation::D90));
    assert_eq!(1, n3.rotated_overlap(&n4, Rotation::D90));
    assert_eq!(1, n4.rotated_overlap(&n4, Rotation::D90));

    //  11          3
    //  O     O    O3  44O
    // 11    22
    assert_eq!(5, n1.rotated_overlap(&n1, Rotation::D180));
    assert_eq!(3, n1.rotated_overlap(&n2, Rotation::D180));
    assert_eq!(2, n1.rotated_overlap(&n3, Rotation::D180));
    assert_eq!(1, n1.rotated_overlap(&n4, Rotation::D180));
    assert_eq!(1, n2.rotated_overlap(&n2, Rotation::D180));
    assert_eq!(2, n2.rotated_overlap(&n3, Rotation::D180));
    assert_eq!(1, n2.rotated_overlap(&n4, Rotation::D180));
    assert_eq!(1, n3.rotated_overlap(&n3, Rotation::D180));
    assert_eq!(2, n3.rotated_overlap(&n4, Rotation::D180));
    assert_eq!(1, n4.rotated_overlap(&n4, Rotation::D180));

    //                4
    // 1     2        4
    // 1O1   2O   O   O
    //   1        33
    assert_eq!(1, n1.rotated_overlap(&n1, Rotation::D270));
    assert_eq!(1, n1.rotated_overlap(&n2, Rotation::D270));
    assert_eq!(2, n1.rotated_overlap(&n3, Rotation::D270));
    assert_eq!(2, n1.rotated_overlap(&n4, Rotation::D270));
    assert_eq!(1, n2.rotated_overlap(&n2, Rotation::D270));
    assert_eq!(2, n2.rotated_overlap(&n3, Rotation::D270));
    assert_eq!(1, n2.rotated_overlap(&n4, Rotation::D270));
    assert_eq!(1, n3.rotated_overlap(&n3, Rotation::D270));
    assert_eq!(1, n3.rotated_overlap(&n4, Rotation::D270));
    assert_eq!(1, n4.rotated_overlap(&n4, Rotation::D270));
}

#[test]
fn test_constraint_before() {
    let g = Grid::new(3, 3);
    let mut n = Neighbourhood::new_around(1, 1, &g);
    let e1 = Neighbourhood {
        offset_x: 1,
        offset_y: 1,
        contents: vec![
            (0, -1, Panel::default()),
            (-1, 0, Panel::default()),
            (0, 0, Panel::default()),
            (1, 0, Panel::default()),
            (0, 1, Panel::default()),
        ],
    };
    assert!(n.same_shape(&e1));
    n.constrain_to_before(1, 2);

    assert!(n.same_shape(&e1));
    n.constrain_to_before(2, 1);

    let e2 = Neighbourhood {
        offset_x: 1,
        offset_y: 1,
        contents: vec![
            (0, -1, Panel::default()),
            (-1, 0, Panel::default()),
            (0, 0, Panel::default()),
            (1, 0, Panel::default()),
        ],
    };

    assert!(n.same_shape(&e2));
    n.constrain_to_before(1, 1);
    let e3 = Neighbourhood {
        offset_x: 1,
        offset_y: 1,
        contents: vec![
            (0, -1, Panel::default()),
            (-1, 0, Panel::default()),
            (0, 0, Panel::default()),
        ],
    };
    assert!(n.same_shape(&e3));
    n.constrain_to_before(0, 1);
    let e4 = Neighbourhood {
        offset_x: 1,
        offset_y: 1,
        contents: vec![(0, -1, Panel::default()), (-1, 0, Panel::default())],
    };
    assert!(n.same_shape(&e4));
    let e5 = Neighbourhood {
        offset_x: 1,
        offset_y: 1,
        contents: vec![(0, -1, Panel::default())],
    };
    n.constrain_to_before(1, 0);
    assert!(n.same_shape(&e5));
}

#[test]
fn neighbour_test2() {
    let g = Grid::new(3, 3);
    let n = Neighbourhood::new_around(0, 0, &g);
    assert_eq!(3, n.contents.len());
    let n = Neighbourhood::new_around(2, 2, &g);
    assert_eq!(3, n.contents.len());
    let n = Neighbourhood::new_around(1, 0, &g);
    assert_eq!(4, n.contents.len());
}
