#![no_std]

pub mod decode;
pub mod font;
pub mod instruction;
pub mod machine;
pub mod screen;

// NOTE: I might want to use a bitfield instead
pub enum Flags {
    Nothing,
    Overflow,
    Underflow,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(pub u16);

impl core::ops::Add<u16> for Address {
    type Output = Address;

    fn add(self, rhs: u16) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl core::ops::AddAssign<u16> for Address {
    fn add_assign(&mut self, rhs: u16) {
        *self = *self + rhs;
    }
}

impl core::ops::Sub<u16> for Address {
    type Output = Address;

    fn sub(self, rhs: u16) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl core::ops::SubAssign<u16> for Address {
    fn sub_assign(&mut self, rhs: u16) {
        *self = *self - rhs;
    }
}

impl core::ops::Index<Address> for [u8] {
    type Output = u8;

    fn index(&self, index: Address) -> &Self::Output {
        self.index(usize::from(index.0))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegIdent(u8);

impl RegIdent {
    pub const F: Self = Self(0xF);

    pub fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for RegIdent {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= 0xF {
            Ok(Self(value))
        } else {
            Err(())
        }
    }
}
