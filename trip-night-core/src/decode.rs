use core::fmt;

use crate::instruction::{Instruction, InstructionSet, OpCode};

#[derive(Clone, Copy, Debug)]
pub struct UnknownInstructionError;

impl fmt::Display for UnknownInstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Attempted to decode an unknown instruction")
    }
}

pub fn decode_instruction(set: &InstructionSet, op: OpCode) -> Result<&dyn Instruction, UnknownInstructionError> {
    use crate::instruction::*;

    let instruction = match op.get_first_nibble() {
        0x0 => match op.get_inner() {
            0x00E0 => set[OP_00E0],
            0x00EE => set[OP_00EE],
            _ => return Err(UnknownInstructionError),
        },

        0x1 => set[OP_1NNN],

        0x2 => set[OP_2NNN],

        0x3 => set[OP_3XNN],

        0x4 => set[OP_4XNN],

        0x5 => set[OP_5XY0],

        0x6 => set[OP_6XNN],

        0x7 => set[OP_7XNN],

        0x8 => match op.get_n() {
            0x0 => set[OP_8XY0],
            0x1 => set[OP_8XY1],
            0x2 => set[OP_8XY2],
            0x3 => set[OP_8XY3],
            0x4 => set[OP_8XY4],
            0x5 => set[OP_8XY5],
            0x6 => set[OP_8XY6],
            0x7 => set[OP_8XY7],
            0xE => set[OP_8XYE],
            _ => return Err(UnknownInstructionError),
        },

        0x9 => set[OP_9XY0],

        0xA => set[OP_ANNN],

        0xB => set[OP_BNNN],

        0xC => set[OP_CXNN],

        0xD => set[OP_DXYN],

        0xE => match op.get_nn() {
            0x9E => set[OP_EX9E],
            0xA1 => set[OP_EXA1],
            _ => return Err(UnknownInstructionError),
        },

        0xF => match op.get_nn() {
            0x07 => set[OP_FX07],
            0x0A => set[OP_FX0A],
            0x15 => set[OP_FX15],
            0x18 => set[OP_FX18],
            0x1E => set[OP_FX1E],
            0x29 => set[OP_FX29],
            0x33 => set[OP_FX33],
            0x55 => set[OP_FX55],
            0x65 => set[OP_FX65],
            _ => return Err(UnknownInstructionError),
        },

        _ => unreachable!("a possible value for the most significant nibble is not handled; this is a bug"),
    };

    Ok(instruction)
}

pub trait DecodeOpCode {
    fn decode(op: OpCode) -> Self;
}
