use core::fmt;

#[derive(Clone, Default, Debug)]
pub struct Screen {
    inner: [u64; 32],
    changed: bool,
}

impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.into_iter().try_for_each(|row| writeln!(f, "{row:064b}"))?;
        Ok(())
    }
}

impl Screen {
    const MSB_ONLY: u8 = 0x1 << 7;

    pub fn clear(&mut self) {
        self.inner.iter_mut().for_each(|row| *row = 0);
        self.changed = true;
    }

    pub fn is_changed(&self) -> bool {
        self.changed
    }

    pub fn reset_changed_flag(&mut self) {
        self.changed = false;
    }

    pub fn set_pixel(&mut self, x: u8, y: u8) {
        self.set_vectored(Self::MSB_ONLY, x, y);
    }

    pub fn unset_pixel(&mut self, x: u8, y: u8) {
        self.unset_vectored(Self::MSB_ONLY, x, y);
    }

    pub fn flip_pixel(&mut self, x: u8, y: u8) -> FlipResult {
        self.flip_vectored(Self::MSB_ONLY, x, y)
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> PixelState {
        let (x, y) = Self::clamp(x, y);
        let mask = Self::generate_mask(Self::MSB_ONLY, x);

        if (self.inner[usize::from(y)] & mask) == 0 {
            PixelState::Unset
        } else {
            PixelState::Set
        }
    }

    pub fn set_vectored(&mut self, vector: u8, x: u8, y: u8) {
        let (x, y) = Self::clamp(x, y);
        let mask = Self::generate_mask(vector, x);

        self.inner[usize::from(y)] |= mask;
        self.changed = true;
    }

    pub fn unset_vectored(&mut self, vector: u8, x: u8, y: u8) {
        let (x, y) = Self::clamp(x, y);
        let mask = Self::generate_mask(vector, x);

        self.inner[usize::from(y)] &= mask;
        self.changed = true;
    }

    pub fn flip_vectored(&mut self, vector: u8, x: u8, y: u8) -> FlipResult {
        let (x, y) = Self::clamp(x, y);
        let mask = Self::generate_mask(vector, x);

        let no_overlap = self.inner[usize::from(y)] & mask == 0;
        self.inner[usize::from(y)] ^= mask;
        self.changed = true;

        if no_overlap {
            FlipResult::NoUnsetBit
        } else {
            FlipResult::UnsetBit
        }
    }

    pub fn get_vectored(&self, x: u8, y: u8) -> u8 {
        let (x, y) = Self::clamp(x, y);
        let mask = Self::generate_mask(0xFF, x);

        ((self.inner[usize::from(y)] & mask) >> (56 - x)).try_into().unwrap()
    }

    pub fn iter(&self) -> ScreenIter<'_> {
        ScreenIter {
            screen: self,
            x: 0,
            y: 0,
        }
    }

    fn clamp(x: u8, y: u8) -> (u8, u8) {
        (x & 0x3F, y & 0x1F)
    }

    fn generate_mask(vector: u8, x: u8) -> u64 {
        u64::from_be_bytes([vector, 0, 0, 0, 0, 0, 0, 0])
            .overflowing_shr(u32::from(x))
            .0
    }
}

// TODO: iteration on set pixels only

pub struct ScreenIter<'a> {
    screen: &'a Screen,
    x: u8,
    y: u8,
}

impl<'a> Iterator for ScreenIter<'a> {
    type Item = (u8, u8, PixelState);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= 64 {
            self.x = 0;
            self.y += 1;
        }

        if self.y >= 32 {
            return None;
        }

        let x = self.x;
        let y = self.y;
        let pixel_state = self.screen.get_pixel(x, y);

        self.x += 1;

        Some((x, y, pixel_state))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PixelState {
    Set,
    Unset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FlipResult {
    UnsetBit,
    NoUnsetBit,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_screen_with_single_row(y: usize, row: u64) -> Screen {
        let mut screen = Screen::default();
        screen.inner[y] = row;
        screen
    }

    #[test]
    fn mask_generation() {
        assert_eq!(Screen::generate_mask(Screen::MSB_ONLY, 10), 0x0020_0000_0000_0000);
        assert_eq!(Screen::generate_mask(Screen::MSB_ONLY, 11), 0x0010_0000_0000_0000);
        assert_eq!(Screen::generate_mask(0xC0, 10), 0x0030_0000_0000_0000);
        assert_eq!(Screen::generate_mask(0xFF, 32), 0x0000_0000_FF00_0000);
        assert_eq!(Screen::generate_mask(0xBE, 40), 0x0000_0000_00BE_0000);
    }

    #[test]
    fn pixel_flip() {
        let mut screen = new_screen_with_single_row(4, 0xDEAD_BEEF_0000_0123);
        assert_eq!(screen.get_pixel(55, 4), PixelState::Set);
        assert_eq!(screen.is_changed(), false);
        assert_eq!(screen.flip_pixel(55, 4), FlipResult::UnsetBit);
        assert_eq!(screen.is_changed(), true);
        assert_eq!(screen.get_pixel(55, 4), PixelState::Unset);
        assert_eq!(screen.inner[4], 0xDEAD_BEEF_0000_0023);
        assert_eq!(screen.flip_pixel(55, 4), FlipResult::NoUnsetBit);
    }

    #[test]
    fn vectored_flip() {
        let mut screen = new_screen_with_single_row(17, 0xDEAD_BEEF_0000_0123);
        assert_eq!(screen.get_vectored(8, 17), 0xAD);
        assert_eq!(screen.is_changed(), false);
        assert_eq!(screen.flip_vectored(0xAD, 8, 17), FlipResult::UnsetBit);
        assert_eq!(screen.is_changed(), true);
        assert_eq!(screen.get_vectored(8, 17), 0x00);
        assert_eq!(screen.inner[17], 0xDE00_BEEF_0000_0123);
        assert_eq!(screen.flip_vectored(0xED, 8, 17), FlipResult::NoUnsetBit);
        assert_eq!(screen.get_vectored(8, 17), 0xED);
    }

    #[test]
    fn clear_screen() {
        let mut screen = new_screen_with_single_row(17, 0xDEAD_BEEF_0000_0123);
        assert_eq!(screen.is_changed(), false);
        screen.clear();
        assert_eq!(screen.inner[17], 0);
        assert_eq!(screen.is_changed(), true);
    }
}
