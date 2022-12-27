use crate::decode::decode_instruction;
use crate::instruction::{InstructionSet, OpCode};
use crate::screen::Screen;
use crate::{Address, Flags, RegIdent};

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

    pub fn reg_add(&mut self, reg: RegIdent, value: u8) -> Flags {
        let reg_idx = usize::from(reg.get());
        self.registers[reg_idx] = self.registers[reg_idx].wrapping_add(value);
        Flags::Nothing // FIXME
    }

    pub fn reg_sub(&mut self, reg: RegIdent, value: u8) -> Flags {
        let reg_idx = usize::from(reg.get());
        self.registers[reg_idx] = self.registers[reg_idx].wrapping_sub(value);
        Flags::Nothing // FIXME
    }
}
