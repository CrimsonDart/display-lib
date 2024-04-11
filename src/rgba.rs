use std::error::Error;
use std::fmt::{Formatter, Display};

use std::ops::{Index, IndexMut};
use std::str::Chars;

use std::num::TryFromIntError;

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Color {
    Red = 2,
    Green = 1,
    Blue = 0,
    Alpha = 3
}

#[derive(Clone, Copy)]
pub union Rgba {
    value: [u8; 4],
    number: u32
}

impl Rgba {

    pub const BLACK: Rgba = Rgba::new_opaque(0, 0, 0);
    pub const WHITE: Rgba = Rgba::new_opaque(255, 255, 255);

    pub const GRAY: Rgba = Rgba::new_opaque(0xAA, 0xAA, 0xAA);
    pub const DARK_GRAY: Rgba = Rgba::new_opaque(55, 55, 55);

    // Primary Additive Colors
    pub const RED: Rgba = Rgba::new_opaque(255, 0, 0);
    pub const GREEN: Rgba = Rgba::new_opaque(0, 255, 0);
    pub const BLUE: Rgba = Rgba::new_opaque(0, 0, 255);

    // Primary Subtractive Colors
    pub const MAGENTA: Rgba = Rgba::new_opaque(255, 0, 255);
    pub const CYAN: Rgba = Rgba::new_opaque(0, 255, 255);
    pub const YELLOW: Rgba = Rgba::new_opaque(255, 255, 0);

    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            value: [blue, green, red, alpha]
        }
    }

    pub const fn new_opaque(red: u8, green: u8, blue: u8) -> Self {
        Self {
            value: [blue, green, red, 255]
        }
    }

    pub fn blend(mut self, rhs: Self, proportion: u8) -> Self {
        use Color::*;

        for c in [Red, Blue, Green] {
            self[c] = blend_color(self[c], rhs[c], proportion);
        }

        self
    }
}

fn blend_color(a: u8, b: u8, t: u8) -> u8 {

    let (a, b, t) = (a as u16, b as u16, t as u16);
    (((b * t) + (a * (255 - t)) + 1) >> 8) as u8
}

impl Index<Color> for Rgba {
    type Output = u8;
    fn index(&self, index: Color) -> &Self::Output {
        unsafe {
            &self.value[index as usize]
        }
    }
}

impl IndexMut<Color> for Rgba {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        unsafe {
            &mut self.value[index as usize]
        }
    }
}

impl Index<usize> for Rgba {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            &self.value[index]
        }
    }
}

impl IndexMut<usize> for Rgba {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe {
            &mut self.value[index]
        }
    }
}

impl Display for Rgba {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        let mut is_first = true;
        for b in unsafe {self.value} {
            if is_first {
                is_first = false;
                write!(f, "{b}")?;
            } else {
                write!(f, ", {b}")?;
            }
        }
        Ok(())
    }
}

impl From<Rgba> for u32 {
    fn from(value: Rgba) -> Self {
        unsafe {
            value.number
        }
    }
}

impl From<u32> for Rgba {
    fn from(value: u32) -> Self {
        Self {
            number: value
        }
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Self::new(0, 0, 0, 255)
    }
}

impl PartialEq for Rgba {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            self.number == other.number
        }
    }
}

impl std::fmt::Debug for Rgba {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(f, "{:?}", self.value)
        }
    }
}

impl Eq for Rgba {}

impl<'a> TryFrom<Chars<'a>> for Rgba {
    type Error = CharsToRgbaError;

    fn try_from(mut value: Chars) -> Result<Self, Self::Error> {
        let num = {
            let mut out = [0; 4];
            for i in 0..4 {

                let (left, right) = (value.next(), value.next());


                if let (Some(n1), Some(n2)) = (left, right) {
                    if let Some(n) = hex_code_to_u8([n1, n2]) {
                        out[i] = n;
                    } else if i != 3 {
                        return Err(CharsToRgbaError::InvalidStr({
                            let mut s = String::new();
                            s.push(n1);
                            s.push(n2);
                            s
                        }
                        ));
                    } else {
                        out[3] = 0xFF;
                    }
                } else if i != 3 {
                    let len = if left.is_some() {i * 2 + 1} else { i * 2 };

                    return Err(CharsToRgbaError::InsufficientLength(len));
                } else {
                    out[3] = 0xFF;
                }
            }
            out
        };

        Ok(Self::new(num[0], num[1], num[2], num[3]))
    }
}

fn hex_code_to_u4(c: char) -> Option<u8> {
    Some(match c {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'A' | 'a' => 10,
        'B' | 'b' => 11,
        'C' | 'c' => 12,
        'D' | 'd' => 13,
        'E' | 'e' => 14,
        'F' | 'f' => 15,
        _ => {
            return None;
        }
    })
}

fn hex_code_to_u8(chars: [char; 2]) -> Option<u8> {
    Some(hex_code_to_u4(chars[0])? * 16 + hex_code_to_u4(chars[1])?)
}

#[derive(Debug)]
pub enum CharsToRgbaError {
    InvalidStr(String),
    InsufficientLength(usize)
}

impl Display for CharsToRgbaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use CharsToRgbaError::*;

        match self {
            InsufficientLength(n) => {
                write!(f, "Input iterator length was insufficient. Len was {n}, len requred is 6 or 8.")
            },
            InvalidStr(s) => {
                write!(f, "{s} contians characters that are not compatiable with the hex codec, which is 0-9, or A-F.")
            }
        }
    }
}

impl Error for CharsToRgbaError {}

pub enum TomlToRgbaError {
    InsufficientStrLen(usize),
    InvalidStr(String),
    IncorrectArrayType(usize),
    InvalidEntryType,
    IntConversionFail
}

impl From<TryFromIntError> for TomlToRgbaError {
    fn from(_value: TryFromIntError) -> Self {
        TomlToRgbaError::IntConversionFail
    }
}

impl From<CharsToRgbaError> for TomlToRgbaError {
    fn from(value: CharsToRgbaError) -> Self {
        match value {
            CharsToRgbaError::InvalidStr(s) => Self::InvalidStr(s),
            CharsToRgbaError::InsufficientLength(len) => Self::InsufficientStrLen(len)
        }
    }
}
