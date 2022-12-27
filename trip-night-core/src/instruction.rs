use bit_field::BitField as _;

use crate::machine::State;
use crate::{Address, RegIdent};

/// CLS
pub const OP_00E0: usize = 0;
/// RET
pub const OP_00EE: usize = 1;

/// JP addr
pub const OP_1NNN: usize = 2;

/// CALL addr
pub const OP_2NNN: usize = 3;

/// SE Vx, byte
pub const OP_3XNN: usize = 4;

/// SNE Vx, byte
pub const OP_4XNN: usize = 5;

/// SE Vx, Vy
pub const OP_5XY0: usize = 6;

/// LD Vx, byte
pub const OP_6XNN: usize = 7;

/// ADD Vx, byte
pub const OP_7XNN: usize = 8;

/// LD Vx, Vy
pub const OP_8XY0: usize = 9;
/// OR Vx, Vy
pub const OP_8XY1: usize = 10;
/// AND Vx, Vy
pub const OP_8XY2: usize = 11;
/// XOR Vx, Vy
pub const OP_8XY3: usize = 12;
/// ADD Vx, Vy
pub const OP_8XY4: usize = 13;
/// SUB Vx, Vy
pub const OP_8XY5: usize = 14;
/// SHR Vx {, Vy}
pub const OP_8XY6: usize = 15;
/// SUBN Vx, Vy
pub const OP_8XY7: usize = 16;
/// SHL Vx {, Vy}
pub const OP_8XYE: usize = 17;

/// SNE Vx, Vy
pub const OP_9XY0: usize = 18;

/// LD I, addr
pub const OP_ANNN: usize = 19;

/// JP V0, addr
pub const OP_BNNN: usize = 20;

/// RND Vx, byte
pub const OP_CXNN: usize = 21;

/// DRW Vx, Vy, nibble
pub const OP_DXYN: usize = 22;

/// SKP Vx
pub const OP_EX9E: usize = 23;
/// SKNP Vx
pub const OP_EXA1: usize = 24;

/// LD Vx, DT
pub const OP_FX07: usize = 25;
/// LD Vx, K
pub const OP_FX0A: usize = 26;
/// LD DT, Vx
pub const OP_FX15: usize = 27;
/// LD ST, Vx
pub const OP_FX18: usize = 28;
/// ADD I, Vx
pub const OP_FX1E: usize = 29;
/// LD F, Vx
pub const OP_FX29: usize = 30;
/// LD B, Vx
pub const OP_FX33: usize = 31;
/// LD [I], Vx
pub const OP_FX55: usize = 32;
/// LD Vx, [I]
pub const OP_FX65: usize = 33;

#[macro_export]
macro_rules! make_instruction {
    ($impl:path) => {{
        fn wrapper(opcode: $crate::instruction::OpCode, state: &mut $crate::machine::State) {
            let decoded = $crate::decode::DecodeOpCode::decode(opcode);
            $impl(decoded, state)
        }
        &wrapper
    }};
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpCode(u16);

impl OpCode {
    pub fn new(inner: u16) -> Self {
        Self(inner)
    }

    pub fn get_inner(self) -> u16 {
        self.0
    }

    pub fn get_first_nibble(self) -> u8 {
        self.0.get_bits(12..16) as u8
    }

    pub fn get_x(self) -> RegIdent {
        RegIdent::try_from(self.0.get_bits(8..12) as u8).expect("an u8 with only the first nibble set")
    }

    pub fn get_y(self) -> RegIdent {
        RegIdent::try_from(self.0.get_bits(4..8) as u8).expect("an u8 with only the first nibble set")
    }

    pub fn get_n(self) -> u8 {
        self.0.get_bits(0..4) as u8
    }

    pub fn get_nn(self) -> u8 {
        self.0.get_bits(0..8) as u8
    }

    pub fn get_nnn(self) -> Address {
        Address(self.0.get_bits(0..12))
    }
}

pub trait Instruction {
    fn execute(&self, opcode: OpCode, state: &mut State);
}

impl<F> Instruction for F
where
    F: Fn(OpCode, &mut State),
{
    fn execute(&self, opcode: OpCode, state: &mut State) {
        self(opcode, state)
    }
}

pub type InstructionSet = [&'static dyn Instruction; 34];
