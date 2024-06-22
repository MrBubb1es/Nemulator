
use bitfield_struct::bitfield;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ControllerButton {
    #[default]
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

pub struct ControllerUpdate {
    pub button: ControllerButton,
    pub player_id: usize,
    pub pressed: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ControllerReadState {
    button: ControllerButton,
    finished: bool,
}

impl ControllerReadState {
    pub fn new() -> Self {
        ControllerReadState{ button: ControllerButton::A, finished: false }
    }

    pub fn next(self) -> Self {
        if self.finished { return self; }

        match self.button {
            ControllerButton::A => ControllerReadState{button: ControllerButton::B, finished: false},
            ControllerButton::B => ControllerReadState{button: ControllerButton::Select, finished: false},
            ControllerButton::Select => ControllerReadState{button: ControllerButton::Start, finished: false},
            ControllerButton::Start => ControllerReadState{button: ControllerButton::Up, finished: false},
            ControllerButton::Up => ControllerReadState{button: ControllerButton::Down, finished: false},
            ControllerButton::Down => ControllerReadState{button: ControllerButton::Left, finished: false},
            ControllerButton::Left => ControllerReadState{button: ControllerButton::Right, finished: false},
            ControllerButton::Right => ControllerReadState{button: ControllerButton::Right, finished: true},
        }
    }

    pub fn button(&self) -> ControllerButton {
        self.button
    }

    pub fn finished(&self) -> bool {
        self.finished
    }
}

#[bitfield(u8)]
pub struct NesController {
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl NesController {
    /// Takes in a read state of the controller, and returns a 0 or 1 based on
    /// if the button specified by the read state is pressed or not. If the
    /// read state of the button has the finished flag set, the output will
    /// always be 1.
    pub fn read_button(&self, read_state: ControllerReadState) -> u8 {
        if read_state.finished { return 1; }

        match read_state.button {
            ControllerButton::A => if self.a() { 1 } else { 0 },
            ControllerButton::B => if self.b() { 1 } else { 0 },
            ControllerButton::Select => if self.select() { 1 } else { 0 },
            ControllerButton::Start => if self.start() { 1 } else { 0 },
            ControllerButton::Up => if self.up() { 1 } else { 0 },
            ControllerButton::Down => if self.down() { 1 } else { 0 },
            ControllerButton::Left => if self.left() { 1 } else { 0 },
            ControllerButton::Right => if self.right() { 1 } else { 0 },
        }
    }

    pub fn set_button(&mut self, button: ControllerButton, val: bool) {
        match button {
            ControllerButton::A => self.set_a(val),
            ControllerButton::B => self.set_b(val),
            ControllerButton::Select => self.set_select(val),
            ControllerButton::Start => self.set_start(val),
            ControllerButton::Up => self.set_up(val),
            ControllerButton::Down => self.set_down(val),
            ControllerButton::Left => self.set_left(val),
            ControllerButton::Right => self.set_right(val),
        }
    }
}