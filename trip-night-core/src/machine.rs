use core::fmt;

use crate::decode::decode_instruction;
use crate::instruction::{InstructionSet, OpCode};
use crate::screen::Screen;
use crate::{Address, RegIdent};

/// A Chip8 virtual machine
pub struct Machine {
    pub state: State,
    pub instruction_set: InstructionSet,
    pub frequency_hz: usize,
    pub counter: usize,
}

impl Machine {
    pub fn new(game_code: &[u8], instruction_set: InstructionSet, frequency_hz: usize) -> Self {
        Self {
            state: State::new(game_code),
            instruction_set,
            frequency_hz,
            counter: 0,
        }
    }

    pub fn is_beeping(&self) -> bool {
        self.state.sound_timer > 0
    }

    pub fn screen(&self) -> &Screen {
        &self.state.screen
    }

    pub fn cycle(&mut self) {
        self.update_counter();
        self.state.screen.reset_changed_flag();

        let opcode = self.fetch_opcode();
        let instruction = decode_instruction(&self.instruction_set, opcode).unwrap();
        instruction.execute(opcode, &mut self.state);
    }

    pub fn update_counter(&mut self) {
        self.counter += 1;

        // NOTE: this is a rough approximation, timers will decrease a bit faster than they should
        let modulus = core::cmp::max(self.frequency_hz / 60, 1);

        if self.counter % modulus == 0 {
            self.state.delay_timer = self.state.delay_timer.saturating_sub(1);
            self.state.sound_timer = self.state.sound_timer.saturating_sub(1);
        }
    }

    fn fetch_opcode(&mut self) -> OpCode {
        let first = self.state.ram[self.state.pc];
        let second = self.state.ram[self.state.pc + 1];
        let op = u16::from_be_bytes([first, second]);
        self.state.pc += 2;
        OpCode::new(op)
    }
}

impl fmt::Display for Machine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "------------ Machine ------------")?;
        write!(f, "{}", self.state)
    }
}

pub struct State {
    /// Memory: 4 kB (or 4096 bytes) of RAM
    pub ram: [u8; 4096],
    /// Program counter, points at the current instruction in memory
    pub pc: Address,
    // Index register pointing at location of a sprite when drawing
    pub index: Address,
    /// Stores return addresses when calling subroutines
    stack: [Address; 16],
    /// Stack pointer, points to the next available slot in the stack
    stack_pointer: u8,
    /// Delay timer register, will be decremented at a rate of 60 Hz until 0 is reached
    pub delay_timer: u8,
    /// Sound timer register, a "beep" will be produced until it reaches 0
    pub sound_timer: u8,
    /// General-purpose registers
    registers: [u8; 16],
    /// Chip8 Screen
    pub screen: Screen,
}

impl State {
    fn new(game_code: &[u8]) -> Self {
        use crate::font;

        let mut ram = [0; 4096];

        ram[0x50..0x50 + font::STANDARD.len()].copy_from_slice(font::STANDARD);
        ram[0x200..0x200 + game_code.len()].copy_from_slice(game_code);

        Self {
            ram,
            pc: Address(0x200),
            index: Address(0),
            stack: [Address(0); 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
            screen: Screen::default(),
        }
    }
}

impl State {
    pub fn stack_push(&mut self, value: Address) {
        self.stack[usize::from(self.stack_pointer)] = value;
        self.stack_pointer += 1;
    }

    pub fn stack_pop(&mut self) -> Address {
        self.stack_pointer -= 1;
        self.stack[usize::from(self.stack_pointer)]
    }

    pub fn reg_write(&mut self, reg: RegIdent, value: u8) {
        self.registers[usize::from(reg.get())] = value;
    }

    pub fn reg_read(&mut self, reg: RegIdent) -> u8 {
        self.registers[usize::from(reg.get())]
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pc_ram_start = usize::from(self.pc.0);
        let pc_ram_end = pc_ram_start + 4;
        let memory_at_pc = &self.ram[pc_ram_start..pc_ram_end];

        let i_ram_start = usize::from(self.index.0);
        let i_ram_end = i_ram_start + 4;
        let memory_at_index = &self.ram[i_ram_start..i_ram_end];

        writeln!(f, "pc: {}", self.pc)?;
        writeln!(f, "ram[{pc_ram_start:03x}..{pc_ram_end:03x}]: {memory_at_pc:02x?}")?;
        writeln!(f, "i: {}", self.index)?;
        writeln!(f, "ram[{i_ram_start:03x}..{i_ram_end:03x}]: {memory_at_index:02x?}")?;
        writeln!(f, "sp: {}", self.stack_pointer)?;
        writeln!(f, "stack: {:?}", self.stack)?;
        writeln!(f, "dt: {:02x}", self.delay_timer)?;
        writeln!(f, "st: {:02x}", self.sound_timer)?;
        writeln!(f, "registers: {:02x?}", self.registers)?;
        write!(f, "screen:\n{}", self.screen)?;

        Ok(())
    }
}
