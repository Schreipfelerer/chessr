use crate::board::Sq64;
use std::fmt;
use std::ops::{Add, Sub};

impl Sq64 {
    #[inline(always)]
    pub fn from_notation(notation: &[u8]) -> Option<Self> {
        if notation.len() != 2 {
            return None;
        }
        if !(b'a'..=b'h').contains(&notation[0]) {
            return None;
        }
        if !(b'1'..=b'8').contains(&notation[1]) {
            return None;
        }
        Some(Sq64(notation[0] - b'a' + ((notation[1] - b'1') * 8)))
    }
    #[inline(always)]
    pub fn rank(self) -> u8 {
        self.0 >> 3
    }
    #[inline(always)]
    pub fn file(self) -> u8 {
        self.0 & 7
    }
    #[inline(always)]
    pub fn mask(self) -> u64 {
        1 << self.0
    }
    #[inline(always)]
    pub fn is_on_bb(self, bb: u64) -> bool {
        bb >> self.0 & 1 == 1
    }
    #[inline(always)]
    pub fn ind(self) -> usize {
        self.0 as usize
    }
}

impl Add<i8> for Sq64 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: i8) -> Self::Output {
        Sq64(self.0.wrapping_add_signed(rhs))
    }
}

impl Sub<i8> for Sq64 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: i8) -> Self::Output {
        Sq64(self.0.wrapping_sub_signed(rhs))
    }
}

impl fmt::Display for Sq64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            (b'a' + self.file()) as char,
            (b'1' + self.rank()) as char
        )
    }
}