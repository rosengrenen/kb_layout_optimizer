use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    x: usize,
    y: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Keyboard<const N: usize> {
    keys: HashMap<Point, Option<char>>,
    fingers: Vec<Finger>,
}

#[derive(Clone, Debug)]
pub struct Finger {
    keys: Vec<Point>,
    default: Point,
    weights: Vec<(Point, Point, f64)>,
}
