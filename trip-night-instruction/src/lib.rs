#![no_std]

use trip_night_core::decode::DecodeOpCode;
use trip_night_core::instruction::{InstructionSet, OpCode};
use trip_night_core::machine::State;
use trip_night_core::{Address, RegIdent};

pub fn make_noop_set() -> InstructionSet {
    fn noop(_: OpCode, _: &mut State) {}
    [&noop; 34]
}

pub fn make_standard_set() -> InstructionSet {
    use trip_night_core::instruction::*;
    use trip_night_core::make_instruction;

    let mut set = make_noop_set();

    // 0×××
    set[OP_00E0] = make_instruction!(ClearScreen::execute);
    set[OP_00EE] = make_instruction!(Ret::execute);

    // 1×××
    set[OP_1NNN] = make_instruction!(Jump::execute);

    // 2×××
    set[OP_2NNN] = make_instruction!(Call::execute);

    // 3×××
    // set[OP_3XNN] = make_instruction!(SkipEqConst::execute);

    // 4×××
    // set[OP_4XNN] = make_instruction!(SkipNeqConst::execute);

    // 5×××
    // set[OP_5XY0] = make_instruction!(SkipEq::execute);

    // 6×××
    set[OP_6XNN] = make_instruction!(Set::execute);

    // 7×××
    set[OP_7XNN] = make_instruction!(AddConst::execute);

    // 8×××
    set[OP_8XY0] = make_instruction!(Assign::execute);
    // set[OP_8XY1] = make_instruction!(BitOr::execute);
    // set[OP_8XY2] = make_instruction!(BitAnd::execute);
    // set[OP_8XY3] = make_instruction!(BitXor::execute);
    // set[OP_8XY4] = make_instruction!(Add::execute);
    // set[OP_8XY5] = make_instruction!(Sub::execute);
    // set[OP_8XY6] = make_instruction!(ShiftRight::execute);
    // set[OP_8XY7] = make_instruction!(SubN::execute);
    // set[OP_8XYE] = make_instruction!(ShiftLeft::execute);

    // 9×××
    // TODO

    // A×××
    set[OP_ANNN] = make_instruction!(SetIndex::execute);

    // B×××
    // TODO

    // C×××
    // TODO

    // D×××
    set[OP_DXYN] = make_instruction!(Draw::execute);

    // E×××
    // TODO

    // F×××
    // TODO

    set
}

//pub fn make_legacy_set() -> InstructionSet {
//    use trip_night_core::instruction::*;
//    use trip_night_core::make_instruction;
//
//    let mut set = make_standard_set();
//
//    set[OP_8XY6] = make_instruction!(ShiftRightLegacy::execute);
//    set[OP_8XYE] = make_instruction!(ShiftLeftLegacy::execute);
//
//    set
//}

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
            state.reg_write(RegIdent::F, 0x01);
        } else {
            state.reg_write(RegIdent::F, 0x00);
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
pub struct SkipEqConst {
    pub left: RegIdent,
    pub right: u8,
}

/// 4XNN
pub struct SkipNeqConst {
    pub left: RegIdent,
    pub right: u8,
}

/// 5XY0
pub struct SkipEq {
    pub left: RegIdent,
    pub right: RegIdent,
}

/// 9XY0
pub struct SkipNeq {
    pub left: RegIdent,
    pub right: RegIdent,
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
        state.reg_add(self.target, self.value);
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
pub struct BitOr {
    pub left: RegIdent,
    pub right: RegIdent,
}

/// 8XY2
pub struct BitAnd {
    pub left: RegIdent,
    pub right: RegIdent,
}

/// 8XY3
pub struct BitXor {
    pub left: RegIdent,
    pub right: RegIdent,
}

/// 8XY4
pub struct Add {
    pub left: RegIdent,
    pub right: RegIdent,
}

/// 8XY5
pub struct Sub {
    pub left: RegIdent,
    pub right: RegIdent,
}

/// 8XY7
pub struct SubN {
    pub left: RegIdent,
    pub right: RegIdent,
}

/// 8XY6
pub struct ShiftRight {
    pub left: RegIdent,
    pub right: RegIdent,
}

/// 8XY6 (Legacy)
///
/// VX will be set to the value of VY (old behavior)
pub struct ShiftRightLegacy {
    pub left: RegIdent,
    pub right: RegIdent,
}

/// 8XYE
pub struct ShiftLeft {
    pub left: RegIdent,
    pub right: RegIdent,
}

/// 8XYE (Legacy)
///
/// VX will be set to the value of VY (old behavior)
pub struct ShiftLeftLegacy {
    pub left: RegIdent,
    pub right: RegIdent,
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
