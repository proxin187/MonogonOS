const SHIFT_SYMBOLS: [char; 10] = ['!', '"', '#', '¤', '%', '&', '/', '(', ')', '='];
const KEYROWS: [char; 26] = [
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P',
    'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L',
    'Z', 'X', 'C', 'V', 'B', 'N', 'M'
];


enum State {
    Ground,
    Await,
}

pub struct Scancodes {
    prefix: u8,
    shift: bool,
    altgr: bool,
    state: State,
}

impl Scancodes {
    pub const fn new() -> Scancodes {
        Scancodes {
            prefix: 0,
            shift: false,
            altgr: false,
            state: State::Ground,
        }
    }

    fn handle_special(&mut self, byte: u8) -> bool {
        match self.state {
            State::Ground => {
                match byte {
                    0xe0 => self.state = State::Await,
                    0x2a => self.shift = true,
                    0xaa => self.shift = false,
                    _ => return false,
                }
            },
            State::Await => {
                self.state = State::Ground;

                match byte {
                    0x38 => self.altgr = true,
                    0xb8 => self.altgr = false,
                    _ => return false,
                }
            },
        }

        true
    }

    fn keyrow(&mut self, index: usize) -> char {
        if self.shift {
            KEYROWS[index]
        } else {
            KEYROWS[index].to_ascii_lowercase()
        }
    }

    pub fn advance(&mut self, byte: u8) -> Option<char> {
        if !self.handle_special(byte) {
            match byte {
                0x01 => Some('\x1b'),
                0x0e => Some('\x08'),
                0x1c => Some('\n'),
                0x39 => Some(' '),
                0x02..=0x0a => {
                    if self.shift {
                        Some(SHIFT_SYMBOLS[byte as usize - 2])
                    } else {
                        Some(((byte - 1) + 48) as char)
                    }
                },
                0x0b => self.shift.then_some(Some('=')).unwrap_or(Some('0')),
                0x0c => self.shift.then_some(Some('?')).unwrap_or(Some('+')),
                0x35 => self.shift.then_some(Some('_')).unwrap_or(Some('-')),
                0x0d => Some('\\'),
                0x10..=0x19 => Some(self.keyrow(byte as usize - 0x10)),
                0x1e..=0x26 => Some(self.keyrow((byte as usize - 0x1e) + 0xa)),
                0x2c..=0x32 => Some(self.keyrow((byte as usize - 0x2c) + 0x13)),
                _ => None,
            }
        } else {
            None
        }
    }
}


