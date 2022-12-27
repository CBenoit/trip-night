#![no_std]

pub mod decode;
pub mod font;
pub mod instruction;
pub mod machine;
pub mod screen;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(pub u16);

impl core::fmt::Display for Address {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:03x}", self.0)
    }
}

impl core::fmt::Debug for Address {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

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
    pub const V0: Self = Self(0x0);
    pub const V1: Self = Self(0x1);
    pub const V2: Self = Self(0x2);
    pub const V3: Self = Self(0x3);
    pub const V4: Self = Self(0x4);
    pub const V5: Self = Self(0x5);
    pub const V6: Self = Self(0x6);
    pub const V7: Self = Self(0x7);
    pub const V8: Self = Self(0x8);
    pub const V9: Self = Self(0x9);
    pub const VA: Self = Self(0xA);
    pub const VB: Self = Self(0xB);
    pub const VC: Self = Self(0xC);
    pub const VD: Self = Self(0xD);
    pub const VE: Self = Self(0xE);
    pub const VF: Self = Self(0xF);

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
