#![no_std]

use trip_night_core::decode::DecodeOpCode;
use trip_night_core::instruction::{InstructionSet, OpCode};
use trip_night_core::machine::State;
use trip_night_core::{Address, RegIdent};

pub fn make_standard_set() -> InstructionSet {
    use trip_night_core::instruction::*;
    use trip_night_core::make_instruction;

    let mut set = make_nop_set();

    // 0×××
    set[OP_00E0] = make_instruction!(ClearScreen::execute);
    set[OP_00EE] = make_instruction!(Ret::execute);

    // 1×××
    set[OP_1NNN] = make_instruction!(Jump::execute);

    // 2×××
    set[OP_2NNN] = make_instruction!(Call::execute);

    // 3×××
    set[OP_3XNN] = make_instruction!(SkipEqConst::execute);

    // 4×××
    set[OP_4XNN] = make_instruction!(SkipNeqConst::execute);

    // 5×××
    set[OP_5XY0] = make_instruction!(SkipEq::execute);

    // 6×××
    set[OP_6XNN] = make_instruction!(Set::execute);

    // 7×××
    set[OP_7XNN] = make_instruction!(AddConst::execute);

    // 8×××
    set[OP_8XY0] = make_instruction!(Assign::execute);
    set[OP_8XY1] = make_instruction!(BitOr::execute);
    set[OP_8XY2] = make_instruction!(BitAnd::execute);
    set[OP_8XY3] = make_instruction!(BitXor::execute);
    set[OP_8XY4] = make_instruction!(Add::execute);
    set[OP_8XY5] = make_instruction!(Sub::execute);
    set[OP_8XY6] = make_instruction!(ShiftRight::execute);
    set[OP_8XY7] = make_instruction!(SubN::execute);
    set[OP_8XYE] = make_instruction!(ShiftLeft::execute);

    // 9×××
    set[OP_9XY0] = make_instruction!(SkipNeq::execute);

    // A×××
    set[OP_ANNN] = make_instruction!(SetIndex::execute);

    // B×××
    set[OP_BNNN] = make_instruction!(JumpOffset::execute);

    // C×××
    set[OP_CXNN] = make_instruction!(Random::execute);

    // D×××
    set[OP_DXYN] = make_instruction!(Draw::execute);

    // E×××
    // set[OP_EX9E] = TODO
    // set[OP_EXA1] = TODO

    // F×××
    // set[OP_FX07] = TODO
    // set[OP_FX0A] = TODO
    // set[OP_FX15] = TODO
    // set[OP_FX18] = TODO
    // set[OP_FX1E] = TODO
    // set[OP_FX29] = TODO
    // set[OP_FX33] = TODO
    // set[OP_FX55] = TODO
    // set[OP_FX65] = TODO

    set
}

pub fn make_legacy_set() -> InstructionSet {
    use trip_night_core::instruction::*;
    use trip_night_core::make_instruction;

    let mut set = make_standard_set();

    set[OP_8XY6] = make_instruction!(ShiftRightLegacy::execute);
    set[OP_8XYE] = make_instruction!(ShiftLeftLegacy::execute);

    set
}

//=== Display ===//

/// 00E0
///
/// Clear the display.
pub struct ClearScreen;

impl DecodeOpCode for ClearScreen {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_inner(), 0x00E0);
        Self
    }
}

impl ClearScreen {
    pub fn execute(self, state: &mut State) {
        state.screen.clear();
    }
}

/// DXYN
///
/// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
///
/// The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are
/// then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the
/// existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set
/// to 0. If the sprite is positioned so part of it is outside the coordinates of the display,
/// it wraps around to the opposite side of the screen. See instruction 8xy3 for more information
/// on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
pub struct Draw {
    pub x_reg: RegIdent,
    pub y_reg: RegIdent,
    pub height: u8,
}

impl DecodeOpCode for Draw {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0xD);
        Self {
            x_reg: opcode.get_x(),
            y_reg: opcode.get_y(),
            height: opcode.get_n(),
        }
    }
}

impl Draw {
    pub fn execute(self, state: &mut State) {
        use trip_night_core::screen::FlipResult;

        let x = state.reg_read(self.x_reg);
        let y = state.reg_read(self.y_reg);

        let start = usize::from(state.index.0);
        let end = start + usize::from(self.height);

        let mut unset_bit = false;

        for (i, sprite_row) in (0..self.height).zip(state.ram[start..end].iter().cloned()) {
            match state.screen.flip_vectored(sprite_row, x, y + i) {
                FlipResult::UnsetBit => unset_bit = true,
                FlipResult::NoUnsetBit => {}
            }
        }

        if unset_bit {
            state.reg_write(RegIdent::VF, 0x01);
        } else {
            state.reg_write(RegIdent::VF, 0x00);
        }
    }
}

//=== Flow Control ===///

/// 00EE
///
/// Return from a subroutine.
///
/// The interpreter sets the program counter to the address at the top of the stack,
/// then subtracts 1 from the stack pointer.
pub struct Ret;

impl DecodeOpCode for Ret {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_inner(), 0x00EE);
        Self
    }
}

impl Ret {
    pub fn execute(self, state: &mut State) {
        state.pc = state.stack_pop();
    }
}

/// 1NNN
///
/// Jump to location nnn.
///
/// The interpreter sets the program counter to nnn.
pub struct Jump {
    pub addr: Address,
}

impl DecodeOpCode for Jump {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x1);
        Self { addr: opcode.get_nnn() }
    }
}

impl Jump {
    pub fn execute(self, state: &mut State) {
        state.pc = self.addr;
    }
}

/// 2NNN
///
/// Call subroutine at nnn.
///
/// The interpreter increments the stack pointer, then puts the current PC on the top of the
/// stack. The PC is then set to nnn.
pub struct Call {
    pub addr: Address,
}

impl DecodeOpCode for Call {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x2);
        Self { addr: opcode.get_nnn() }
    }
}

impl Call {
    pub fn execute(self, state: &mut State) {
        state.stack_push(state.pc);
        state.pc = self.addr;
    }
}

/// 3XNN
///
/// Skip next instruction if Vx = nn.
///
/// The interpreter compares register Vx to nn, and if they are equal, increments the program
/// counter by 2.
pub struct SkipEqConst {
    pub left: RegIdent,
    pub right: u8,
}

impl DecodeOpCode for SkipEqConst {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x3);
        Self {
            left: opcode.get_x(),
            right: opcode.get_nn(),
        }
    }
}

impl SkipEqConst {
    pub fn execute(self, state: &mut State) {
        let lhs = state.reg_read(self.left);
        let rhs = self.right;

        if lhs == rhs {
            state.pc += 2;
        }
    }
}

/// 4XNN
///
/// Skip next instruction if Vx != nn.
///
/// The interpreter compares register Vx to nn, and if they are not equal, increments the program counter by 2.
pub struct SkipNeqConst {
    pub left: RegIdent,
    pub right: u8,
}

impl DecodeOpCode for SkipNeqConst {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x4);
        Self {
            left: opcode.get_x(),
            right: opcode.get_nn(),
        }
    }
}

impl SkipNeqConst {
    pub fn execute(self, state: &mut State) {
        let lhs = state.reg_read(self.left);
        let rhs = self.right;

        if lhs != rhs {
            state.pc += 2;
        }
    }
}

/// 5XY0
///
/// Skip next instruction if Vx = Vy.
///
/// The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
pub struct SkipEq {
    pub left: RegIdent,
    pub right: RegIdent,
}

impl DecodeOpCode for SkipEq {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x5);
        debug_assert_eq!(opcode.get_n(), 0x0);
        Self {
            left: opcode.get_x(),
            right: opcode.get_y(),
        }
    }
}

impl SkipEq {
    pub fn execute(self, state: &mut State) {
        let lhs = state.reg_read(self.left);
        let rhs = state.reg_read(self.right);

        if lhs == rhs {
            state.pc += 2;
        }
    }
}

/// 9XY0
///
/// Skip next instruction if Vx != Vy.
///
/// The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
pub struct SkipNeq {
    pub left: RegIdent,
    pub right: RegIdent,
}

impl DecodeOpCode for SkipNeq {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x9);
        debug_assert_eq!(opcode.get_n(), 0x0);
        Self {
            left: opcode.get_x(),
            right: opcode.get_y(),
        }
    }
}

impl SkipNeq {
    pub fn execute(self, state: &mut State) {
        let lhs = state.reg_read(self.left);
        let rhs = state.reg_read(self.right);

        if lhs != rhs {
            state.pc += 2;
        }
    }
}

/// BNNN
///
/// Jump to location nnn + V0.
///
/// The program counter is set to nnn plus the value of V0.
pub struct JumpOffset {
    pub addr: Address,
}

impl DecodeOpCode for JumpOffset {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0xB);
        Self { addr: opcode.get_nnn() }
    }
}

impl JumpOffset {
    pub fn execute(self, state: &mut State) {
        let offset = state.reg_read(RegIdent::V0);
        state.pc = self.addr + u16::from(offset);
    }
}

//== Math operations ==//

/// 6XNN
///
/// Set Vx = nn.
///
/// The interpreter puts the value nn into register Vx.
pub struct Set {
    pub target: RegIdent,
    pub value: u8,
}

impl DecodeOpCode for Set {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x6);
        Self {
            target: opcode.get_x(),
            value: opcode.get_nn(),
        }
    }
}

impl Set {
    pub fn execute(self, state: &mut State) {
        state.reg_write(self.target, self.value);
    }
}

/// 7XNN
///
/// Set Vx = Vx + kk.
///
/// Adds the value kk to the value of register Vx, then stores the result in Vx.
pub struct AddConst {
    pub target: RegIdent,
    pub value: u8,
}

impl DecodeOpCode for AddConst {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x7);
        Self {
            target: opcode.get_x(),
            value: opcode.get_nn(),
        }
    }
}

impl AddConst {
    pub fn execute(self, state: &mut State) {
        let lhs = state.reg_read(self.target);
        let rhs = self.value;
        let added = lhs.wrapping_add(rhs);
        state.reg_write(self.target, added);
    }
}

/// 8XY0
///
/// Set Vx = Vy.
///
/// Stores the value of register Vy in register Vx.
pub struct Assign {
    pub target: RegIdent,
    pub from: RegIdent,
}

impl DecodeOpCode for Assign {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x8);
        debug_assert_eq!(opcode.get_n(), 0x0);
        Self {
            target: opcode.get_x(),
            from: opcode.get_y(),
        }
    }
}

impl Assign {
    pub fn execute(self, state: &mut State) {
        let new_val = state.reg_read(self.from);
        state.reg_write(self.target, new_val);
    }
}

/// 8XY1
///
/// Set Vx = Vx OR Vy.
///
/// Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx. A bitwise OR
/// compares the corrseponding bits from two values, and if either bit is 1, then the same bit in
/// the result is also 1. Otherwise, it is 0.
pub struct BitOr {
    pub left: RegIdent,
    pub right: RegIdent,
}

impl DecodeOpCode for BitOr {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x8);
        debug_assert_eq!(opcode.get_n(), 0x1);
        Self {
            left: opcode.get_x(),
            right: opcode.get_y(),
        }
    }
}

impl BitOr {
    pub fn execute(self, state: &mut State) {
        let lhs = state.reg_read(self.left);
        let rhs = state.reg_read(self.right);
        let xored = lhs | rhs;
        state.reg_write(self.left, xored);
    }
}

/// 8XY2
///
/// Set Vx = Vx AND Vy.
///
/// Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx. A bitwise AND
/// compares the corrseponding bits from two values, and if both bits are 1, then the same bit in
/// the result is also 1. Otherwise, it is 0.
pub struct BitAnd {
    pub left: RegIdent,
    pub right: RegIdent,
}

impl DecodeOpCode for BitAnd {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x8);
        debug_assert_eq!(opcode.get_n(), 0x2);
        Self {
            left: opcode.get_x(),
            right: opcode.get_y(),
        }
    }
}

impl BitAnd {
    pub fn execute(self, state: &mut State) {
        let lhs = state.reg_read(self.left);
        let rhs = state.reg_read(self.right);
        let xored = lhs & rhs;
        state.reg_write(self.left, xored);
    }
}

/// 8XY3
///
/// Set Vx = Vx XOR Vy.
///
/// Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx. An
/// exclusive OR compares the corrseponding bits from two values, and if the bits are not both the
/// same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
pub struct BitXor {
    pub left: RegIdent,
    pub right: RegIdent,
}

impl DecodeOpCode for BitXor {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x8);
        debug_assert_eq!(opcode.get_n(), 0x3);
        Self {
            left: opcode.get_x(),
            right: opcode.get_y(),
        }
    }
}

impl BitXor {
    pub fn execute(self, state: &mut State) {
        let lhs = state.reg_read(self.left);
        let rhs = state.reg_read(self.right);
        let xored = lhs ^ rhs;
        state.reg_write(self.left, xored);
    }
}

/// 8XY4
///
/// Set Vx = Vx + Vy, set VF = carry.
///
/// The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,)
/// VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
pub struct Add {
    pub left: RegIdent,
    pub right: RegIdent,
}

impl DecodeOpCode for Add {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x8);
        debug_assert_eq!(opcode.get_n(), 0x4);
        Self {
            left: opcode.get_x(),
            right: opcode.get_y(),
        }
    }
}

impl Add {
    pub fn execute(self, state: &mut State) {
        let lhs = state.reg_read(self.left);
        let rhs = state.reg_read(self.right);
        let (added, overflowed) = lhs.overflowing_add(rhs);

        if overflowed {
            state.reg_write(RegIdent::VF, 0x1);
        } else {
            state.reg_write(RegIdent::VF, 0x0);
        }

        state.reg_write(self.left, added);
    }
}

/// 8XY5
///
/// Set Vx = Vx - Vy, set VF = NOT borrow.
///
/// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results
/// stored in Vx.
pub struct Sub {
    pub left: RegIdent,
    pub right: RegIdent,
}

impl DecodeOpCode for Sub {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x8);
        debug_assert_eq!(opcode.get_n(), 0x5);
        Self {
            left: opcode.get_x(),
            right: opcode.get_y(),
        }
    }
}

impl Sub {
    pub fn execute(self, state: &mut State) {
        let lhs = state.reg_read(self.left);
        let rhs = state.reg_read(self.right);
        let (substracted, underflowed) = lhs.overflowing_sub(rhs);

        if underflowed {
            state.reg_write(RegIdent::VF, 0x0);
        } else {
            state.reg_write(RegIdent::VF, 0x1);
        }

        state.reg_write(self.left, substracted);
    }
}

/// 8XY7
///
/// Set Vx = Vy - Vx, set VF = NOT borrow.
///
/// If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results
/// stored in Vx.
pub struct SubN {
    pub left: RegIdent,
    pub right: RegIdent,
}

impl DecodeOpCode for SubN {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x8);
        debug_assert_eq!(opcode.get_n(), 0x7);
        Self {
            left: opcode.get_x(),
            right: opcode.get_y(),
        }
    }
}

impl SubN {
    pub fn execute(self, state: &mut State) {
        let lhs = state.reg_read(self.left);
        let rhs = state.reg_read(self.right);
        let (substracted, underflowed) = rhs.overflowing_sub(lhs);

        if underflowed {
            state.reg_write(RegIdent::VF, 0x1);
        } else {
            state.reg_write(RegIdent::VF, 0x0);
        }

        state.reg_write(self.left, substracted);
    }
}

/// 8XY6
///
/// Set Vx = Vx SHR 1.
///
/// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided
/// by 2.
pub struct ShiftRight {
    pub left: RegIdent,
    pub right: RegIdent,
}

impl DecodeOpCode for ShiftRight {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x8);
        debug_assert_eq!(opcode.get_n(), 0x6);
        Self {
            left: opcode.get_x(),
            right: opcode.get_y(),
        }
    }
}

impl ShiftRight {
    pub fn execute(self, state: &mut State) {
        let lhs = state.reg_read(self.left);
        // NOTE: here, Vy is left unused

        let (shifted, overflowed) = lhs.overflowing_shr(1);

        if overflowed {
            state.reg_write(RegIdent::VF, 0x1);
        } else {
            state.reg_write(RegIdent::VF, 0x0);
        }

        state.reg_write(self.left, shifted);
    }
}

/// Legacy 8XY6
///
/// Set Vx = Vy SHR 1 (original COSMAC VIP behavior)
pub struct ShiftRightLegacy {
    pub left: RegIdent,
    pub right: RegIdent,
}

impl DecodeOpCode for ShiftRightLegacy {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x8);
        debug_assert_eq!(opcode.get_n(), 0x6);
        Self {
            left: opcode.get_x(),
            right: opcode.get_y(),
        }
    }
}

impl ShiftRightLegacy {
    pub fn execute(self, state: &mut State) {
        let rhs = state.reg_read(self.right);

        let (shifted, overflowed) = rhs.overflowing_shr(1);

        if overflowed {
            state.reg_write(RegIdent::VF, 0x1);
        } else {
            state.reg_write(RegIdent::VF, 0x0);
        }

        state.reg_write(self.left, shifted);
    }
}

/// 8XYE
///
/// Set Vx = Vx SHL 1.
///
/// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is
/// multiplied by 2.
pub struct ShiftLeft {
    pub left: RegIdent,
    pub right: RegIdent,
}

impl DecodeOpCode for ShiftLeft {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x8);
        debug_assert_eq!(opcode.get_n(), 0x6);
        Self {
            left: opcode.get_x(),
            right: opcode.get_y(),
        }
    }
}

impl ShiftLeft {
    pub fn execute(self, state: &mut State) {
        let lhs = state.reg_read(self.left);
        // NOTE: here, Vy is left unused

        let (shifted, overflowed) = lhs.overflowing_shl(1);

        if overflowed {
            state.reg_write(RegIdent::VF, 0x1);
        } else {
            state.reg_write(RegIdent::VF, 0x0);
        }

        state.reg_write(self.left, shifted);
    }
}

/// Legacy 8XYE
///
/// Set Vx = Vy SHL 1.
pub struct ShiftLeftLegacy {
    pub left: RegIdent,
    pub right: RegIdent,
}

impl DecodeOpCode for ShiftLeftLegacy {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0x8);
        debug_assert_eq!(opcode.get_n(), 0x6);
        Self {
            left: opcode.get_x(),
            right: opcode.get_y(),
        }
    }
}

impl ShiftLeftLegacy {
    pub fn execute(self, state: &mut State) {
        let rhs = state.reg_read(self.right);

        let (shifted, overflowed) = rhs.overflowing_shl(1);

        if overflowed {
            state.reg_write(RegIdent::VF, 0x1);
        } else {
            state.reg_write(RegIdent::VF, 0x0);
        }

        state.reg_write(self.left, shifted);
    }
}

/// CXNN
///
///
/// Set Vx = random byte AND nn.
///
/// The interpreter generates a random number from 0 to 255, which is then ANDed with the value nn.
/// The results are stored in Vx. See instruction 8XY2 for more information on AND.
pub struct Random {
    pub target: RegIdent,
    pub mask: u8,
}

impl DecodeOpCode for Random {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0xC);
        Self {
            target: opcode.get_x(),
            mask: opcode.get_nn(),
        }
    }
}

impl Random {
    pub fn execute(self, state: &mut State) {
        let random_number = 0x7A; // TODO: randomness
        let result = random_number & self.mask;
        state.reg_write(self.target, result);
    }
}

//=== Memory ===//

/// ANNN
///
/// Set I = nnn.
///
/// The value of register I is set to nnn.
pub struct SetIndex {
    pub addr: Address,
}

impl DecodeOpCode for SetIndex {
    fn decode(opcode: OpCode) -> Self {
        debug_assert_eq!(opcode.get_first_nibble(), 0xA);
        Self { addr: opcode.get_nnn() }
    }
}

impl SetIndex {
    pub fn execute(self, state: &mut State) {
        state.index = self.addr;
    }
}
