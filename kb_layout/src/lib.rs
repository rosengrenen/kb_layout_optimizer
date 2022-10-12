pub struct Keyboard {
    pub buttons: Vec<Button>,
}

pub struct Button {
    pub hand: Hand,
    pub finger: Finger,
    pub position: Position,
    pub matrix_position: MatrixPosition,
}

pub enum Hand {
    Left,
    Right,
}

pub enum Finger {
    Thumb,
    Pointer,
    Middle,
    Ring,
    Pinky,
}

pub struct Position {
    pub x: f64,
    pub y: f64,
}

pub struct MatrixPosition {
    pub col: u8,
    pub row: u8,
}
