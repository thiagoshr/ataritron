mod instructions;
mod flags;

#[cfg(test)]
mod tests;

use crate::memory::Memory;
use instructions::*;

pub struct Cpu {
    /// program counter
    pc: u16,

    /// accumulator
    a: u8,

    /// general purpose
    x: u8,

    /// general purpose
    y: u8,

    /// stack pointer -- 8bits, stack is on page 0x01
    sp: u8,

    /// status register (flags)
    sr: u8,

    memory: Memory,

    /// how many cycles the cpu needs to complete the running instruction
    cycles_busy: u8
}

impl Cpu {
    pub fn new(mem: Memory) -> Cpu {
        Cpu {
            sp: 0xff, // stack: [0x0100, 0x01ff] 
            pc: 0x1000, // cartridge first address
            a: 0,
            x: 0,
            y: 0,
            sr: 0,
            memory: mem,
            cycles_busy: 0
        }
    }

    fn reset(&mut self) {
        self.sp = 0xff;
        self.pc = self.load_little_endian_u16(0xfffc);
        self.a = 0;
        self.x = 0;
        self.sr = 0;
        self.cycles_busy = 1;
    }

    fn load_little_endian_u16(&self, addr : u16) -> u16 {
        let low_bytes = self.memory.load(addr).unwrap();
        let high_bytes = self.memory.load(addr + 1).unwrap();

        ((high_bytes as u16) << 8) | (low_bytes as u16)
    }

    /// fetches the next instruction to be run and increments the program counter
    fn fetch(&mut self) -> Instruction {
        let opcode = self.memory.load(self.pc).unwrap();
        let instruction_size : u16;

        let instruction = match opcode {
            0x00 => {
                instruction_size = 1;
                Instruction { // BRK
                    operation: Operations::SoftwareInterrupt,
                    addressing: Addressing::Implied,
                    cycle_count: 7
                }
            },
            0x09 => {
                instruction_size = 2;
                Instruction {  // ORA immediate
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::Immediate(self.memory.load(self.pc + 1).unwrap()),
                    cycle_count: 2
                }
            },
            0x05 => {
                instruction_size = 2;
                Instruction { // ORA zeropage
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1).unwrap()),
                    cycle_count: 3
                }
            },
            0x15 => {
                instruction_size = 2;
                Instruction { // ORA indexed zeropage
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1).unwrap(), self.x),
                    cycle_count: 4
                }
            },
            0x0d => {
                instruction_size = 3;
                Instruction { // ORA absolute
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)),
                    cycle_count: 4
                }
            },
            0x1d => {
                instruction_size = 3;
                Instruction { // ORA absolute,X
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1), self.x),
                    cycle_count: 4
                }
            },
            0x19 => {
                instruction_size = 3;
                Instruction { // ORA absolute,Y
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1), self.y),
                    cycle_count: 4
                }
            },
            0x01 => {
                instruction_size = 2;
                Instruction { // ORA (indirect,X)
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::PreindexedIndirect(self.memory.load(self.pc + 1).unwrap(), self.x),
                    cycle_count: 6
                }
            },
            0x11 => {
                instruction_size = 2;
                Instruction { // ORA (indirect), Y
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::PostindexedIndirect(self.memory.load(self.pc + 1).unwrap(), self.y),
                    cycle_count: 5
                }
            },
            0x0a => {
                instruction_size = 1;
                Instruction { // ASL accumulator (implied)
                    operation: Operations::ArithmeticShiftLeft,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0x06 => {
                instruction_size = 2;
                Instruction { // ASL zeropage 
                    operation: Operations::ArithmeticShiftLeft,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1).unwrap()),
                    cycle_count: 5
                }
            },
            0x16 => {
                instruction_size = 2;
                Instruction { /// ASL zeropage,X
                    operation: Operations::ArithmeticShiftLeft,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1).unwrap(), self.x),
                    cycle_count: 6
                }
            },
            0x0e => {
                instruction_size = 3;
                Instruction { // ASL absolute
                    operation: Operations::ArithmeticShiftLeft,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)),
                    cycle_count: 6
                }
            },
            0x1e => {
                instruction_size = 3;
                Instruction { // ASL absolute, X
                    operation: Operations::ArithmeticShiftLeft,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1), self.x),
                    cycle_count: 7
                }
            },
            0x08 => {
                instruction_size = 1;
                Instruction { // PHP implied
                    operation: Operations::PushStatusRegister,
                    addressing: Addressing::Implied,
                    cycle_count: 3
                }
            },
            0x10 => {
                instruction_size = 2;
                Instruction { // BPL relative
                    operation: Operations::BranchOnPlus,
                    addressing: Addressing::RelativeAddress(self.memory.load(self.pc + 1).unwrap()),
                    cycle_count: 2
                }
            },
            0x18 => {
                instruction_size = 1;
                Instruction { // CLC implied
                    operation: Operations::ClearCarry,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0x20 => {
                instruction_size = 3;
                Instruction { // JSR absolute
                    operation: Operations::JumpSubroutine,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)),
                    cycle_count: 6
                }
            },
            0x29 => {
                instruction_size = 2;
                Instruction { // AND immediate
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::Immediate(self.memory.load(self.pc + 1).unwrap()),
                    cycle_count: 2
                }
            },
            0x25 => {
                instruction_size = 2;
                Instruction { // AND zeropage
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1).unwrap()),
                    cycle_count: 3
                }
            },
            0x35 => {
                instruction_size = 2;
                Instruction { // AND zeropage,X
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1).unwrap(), self.x),
                    cycle_count: 4
                }
            },
            0x2d => {
                instruction_size = 3;
                Instruction { // AND absolute
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)),
                    cycle_count: 4
                }
            },
            0x3d => {
                instruction_size = 3;
                Instruction { // AND absolute,X
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1), self.x),
                    cycle_count: 4
                }
            },
            0x39 => {
                instruction_size = 3;
                Instruction { // AND absolute,Y
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1), self.y),
                    cycle_count: 4
                }
            },
            0x21 => {
                instruction_size = 2;
                Instruction { // AND (indirect,X)
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::PreindexedIndirect(self.memory.load(self.pc + 1).unwrap(), self.x),
                    cycle_count: 6
                }
            },
            0x31 => {
                instruction_size = 2;
                Instruction { // AND (indirect),Y
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::PostindexedIndirect(self.memory.load(self.pc + 1).unwrap(), self.y),
                    cycle_count: 5
                }
            },
            0x24 => {
                instruction_size = 2;
                Instruction { // BIT zeropage
                    operation: Operations::BitTest,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1).unwrap()),
                    cycle_count: 3
                }
            },
            0x2c => {
                instruction_size = 3;
                Instruction { // BIT absolute
                    operation: Operations::BitTest,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)),
                    cycle_count: 4
                }
            },
            0x2a => {
                instruction_size = 1;
                Instruction { // ROL accumulator
                    operation: Operations::RotateLeft,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0x26 => {
                instruction_size = 2;
                Instruction { // ROL zeropage
                    operation: Operations::RotateLeft,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1).unwrap()),
                    cycle_count: 5
                }
            },
            0x36 => {
                instruction_size = 2;
                Instruction { // ROL zeropage, X
                    operation: Operations::RotateLeft,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1).unwrap(), self.x),
                    cycle_count: 6
                }
            },
            0x2e => {
                instruction_size = 3;
                Instruction { // ROL absolute
                    operation: Operations::RotateLeft,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)),
                    cycle_count: 6
                }
            },
            0x3e => {
                instruction_size = 3;
                Instruction { // ROL absolute, X
                    operation: Operations::RotateLeft,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1), self.x),
                    cycle_count: 7
                }
            },
            0x30 => {
                instruction_size = 2;
                Instruction { // BMI relative
                    operation: Operations::BranchOnMinus,
                    addressing: Addressing::RelativeAddress(self.memory.load(self.pc + 1).unwrap()),
                    cycle_count: 2
                }
            },
            0x38 => {
                instruction_size = 1;
                Instruction { // SEC
                    operation: Operations::SetCarry,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0x40 => {
                instruction_size = 1;
                Instruction { // RTI 
                    operation: Operations::ReturnFromInterrupt,
                    addressing: Addressing::Implied,
                    cycle_count: 6
                }
            },
            _ => {
                instruction_size = 1;
                Instruction {
                    operation: Operations::NoOperation,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            }
        };

        self.pc += instruction_size;
        instruction
    }
}
