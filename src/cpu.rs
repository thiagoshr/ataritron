mod instructions;
mod flags;

#[cfg(test)]
mod tests;

use crate::memory::{Memory, OutOfRangeError};
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
        self.pc = self.load_little_endian_u16(0xfffc).expect("Error: unexpected end of memory");
        self.a = 0;
        self.x = 0;
        self.sr = 0;
        self.cycles_busy = 1;
    }

    fn load_little_endian_u16(&self, addr : u16) -> Result<u16, OutOfRangeError> {
        let low_bytes = self.memory.load(addr)?;
        let high_bytes = self.memory.load(addr + 1)?;

        Ok(((high_bytes as u16) << 8) | (low_bytes as u16))
    }

    /// fetches the next instruction to be run and increments the program counter
    fn fetch(&mut self) -> Result<Instruction, OutOfRangeError> {
        let opcode = self.memory.load(self.pc)?;
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
                    addressing: Addressing::Immediate(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0x05 => {
                instruction_size = 2;
                Instruction { // ORA zeropage
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0x15 => {
                instruction_size = 2;
                Instruction { // ORA indexed zeropage
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0x0d => {
                instruction_size = 3;
                Instruction { // ORA absolute
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 4
                }
            },
            0x1d => {
                instruction_size = 3;
                Instruction { // ORA absolute,X
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0x19 => {
                instruction_size = 3;
                Instruction { // ORA absolute,Y
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.y),
                    cycle_count: 4
                }
            },
            0x01 => {
                instruction_size = 2;
                Instruction { // ORA (indirect,X)
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::PreindexedIndirect(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 6
                }
            },
            0x11 => {
                instruction_size = 2;
                Instruction { // ORA (indirect), Y
                    operation: Operations::InclusiveOrWithAccumulator,
                    addressing: Addressing::PostindexedIndirect(self.memory.load(self.pc + 1)?, self.y),
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
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 5
                }
            },
            0x16 => {
                instruction_size = 2;
                Instruction { /// ASL zeropage,X
                    operation: Operations::ArithmeticShiftLeft,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 6
                }
            },
            0x0e => {
                instruction_size = 3;
                Instruction { // ASL absolute
                    operation: Operations::ArithmeticShiftLeft,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 6
                }
            },
            0x1e => {
                instruction_size = 3;
                Instruction { // ASL absolute, X
                    operation: Operations::ArithmeticShiftLeft,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.x),
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
                    addressing: Addressing::RelativeAddress(self.memory.load(self.pc + 1)?),
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
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 6
                }
            },
            0x29 => {
                instruction_size = 2;
                Instruction { // AND immediate
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::Immediate(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0x25 => {
                instruction_size = 2;
                Instruction { // AND zeropage
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0x35 => {
                instruction_size = 2;
                Instruction { // AND zeropage,X
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0x2d => {
                instruction_size = 3;
                Instruction { // AND absolute
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 4
                }
            },
            0x3d => {
                instruction_size = 3;
                Instruction { // AND absolute,X
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0x39 => {
                instruction_size = 3;
                Instruction { // AND absolute,Y
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.y),
                    cycle_count: 4
                }
            },
            0x21 => {
                instruction_size = 2;
                Instruction { // AND (indirect,X)
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::PreindexedIndirect(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 6
                }
            },
            0x31 => {
                instruction_size = 2;
                Instruction { // AND (indirect),Y
                    operation: Operations::AndWithAccumulator,
                    addressing: Addressing::PostindexedIndirect(self.memory.load(self.pc + 1)?, self.y),
                    cycle_count: 5
                }
            },
            0x24 => {
                instruction_size = 2;
                Instruction { // BIT zeropage
                    operation: Operations::BitTest,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0x2c => {
                instruction_size = 3;
                Instruction { // BIT absolute
                    operation: Operations::BitTest,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
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
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 5
                }
            },
            0x36 => {
                instruction_size = 2;
                Instruction { // ROL zeropage, X
                    operation: Operations::RotateLeft,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 6
                }
            },
            0x2e => {
                instruction_size = 3;
                Instruction { // ROL absolute
                    operation: Operations::RotateLeft,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 6
                }
            },
            0x3e => {
                instruction_size = 3;
                Instruction { // ROL absolute, X
                    operation: Operations::RotateLeft,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.x),
                    cycle_count: 7
                }
            },
            0x30 => {
                instruction_size = 2;
                Instruction { // BMI relative
                    operation: Operations::BranchOnMinus,
                    addressing: Addressing::RelativeAddress(self.memory.load(self.pc + 1)?),
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
            0x49 => {
                instruction_size = 2;
                Instruction { // EOR immediate
                    operation: Operations::ExclusiveOrWithAccumulator,
                    addressing: Addressing::Immediate(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0x45 => {
                instruction_size = 2;
                Instruction { // EOR zeropage
                    operation: Operations::ExclusiveOrWithAccumulator,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0x55 => {
                instruction_size = 2;
                Instruction { // EOR zeropage, X
                    operation: Operations::ExclusiveOrWithAccumulator,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0x4d => {
                instruction_size = 3;
                Instruction { // EOR absolute
                    operation: Operations::ExclusiveOrWithAccumulator,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 4
                }
            },
            0x5d => {
                instruction_size = 3;
                Instruction { // EOR absolute,X
                    operation: Operations::ExclusiveOrWithAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0x59 => {
                instruction_size = 3;
                Instruction { // EOR absolute,Y
                    operation: Operations::ExclusiveOrWithAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.y),
                    cycle_count: 4
                }
            },
            0x41 => {
                instruction_size = 2;
                Instruction { // EOR (indirect,X)
                    operation: Operations::ExclusiveOrWithAccumulator,
                    addressing: Addressing::PreindexedIndirect(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 6
                }
            },
            0x51 => {
                instruction_size = 2;
                Instruction { // EOR (indirect),Y
                    operation: Operations::ExclusiveOrWithAccumulator,
                    addressing: Addressing::PostindexedIndirect(self.memory.load(self.pc + 1)?, self.y),
                    cycle_count: 5
                }
            },
            0x4a => {
                instruction_size = 1;
                Instruction { // LSR accumulator
                    operation: Operations::LogicalShiftRight,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0x46 => {
                instruction_size = 2;
                Instruction { // LSR zeropage
                    operation: Operations::LogicalShiftRight,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 5
                }
            },
            0x56 => {
                instruction_size = 2;
                Instruction { // LSR zeropage,X
                    operation: Operations::LogicalShiftRight,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 6
                }
            },
            0x4e => {
                instruction_size = 3;
                Instruction { // LSR absolute
                    operation: Operations::LogicalShiftRight,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 6
                }
            },
            0x5e => {
                instruction_size = 3;
                Instruction { // LSR absolute,X
                    operation: Operations::LogicalShiftRight,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.x),
                    cycle_count: 7
                }
            },
            0x4c => {
                instruction_size = 3;
                Instruction { // JMP absolute
                    operation: Operations::Jump,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0x6c => {
                instruction_size = 3;
                Instruction { // JMP indirect
                    operation: Operations::Jump,
                    addressing: Addressing::Indirect(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 5
                }
            },
            0x50 => {
                instruction_size = 2;
                Instruction { // BVC relative
                    operation: Operations::BranchOnOverflowClear,
                    addressing: Addressing::RelativeAddress(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0x58 => {
                instruction_size = 1;
                Instruction { // CLI
                    operation: Operations::ClearInterruptDisable,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0x60 => {
                instruction_size = 1;
                Instruction { // RTS
                    operation: Operations::ReturnFromSubroutine,
                    addressing: Addressing::Implied,
                    cycle_count: 6
                }
            },
            0x69 => {
                instruction_size = 2;
                Instruction { // ADC immediate
                    operation: Operations::AddWithCarry,
                    addressing: Addressing::Immediate(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0x65 => {
                instruction_size = 2;
                Instruction { // ADC zeropage
                    operation: Operations::AddWithCarry,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0x75 => {
                instruction_size = 2;
                Instruction { // ADC zeropage,X
                    operation: Operations::AddWithCarry,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0x6d => {
                instruction_size = 3;
                Instruction { // ADC absolute
                    operation: Operations::AddWithCarry,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 4
                }
            },
            0x7d => {
                instruction_size = 3;
                Instruction { // ADC absolute,X
                    operation: Operations::AddWithCarry,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0x79 => {
                instruction_size = 3;
                Instruction { // ADC absolute,Y
                    operation: Operations::AddWithCarry,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.y),
                    cycle_count: 4
                }
            },
            0x61 => {
                instruction_size = 2;
                Instruction { // ADC (indirect,X)
                    operation: Operations::AddWithCarry,
                    addressing: Addressing::PreindexedIndirect(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 6
                }
            },
            0x71 => {
                instruction_size = 2;
                Instruction { // ADC (indirect),Y
                    operation: Operations::AddWithCarry,
                    addressing: Addressing::PostindexedIndirect(self.memory.load(self.pc + 1)?, self.y),
                    cycle_count: 5
                }
            },
            0x6a => {
                instruction_size = 1;
                Instruction { // ROR accumulator
                    operation: Operations::RotateRight,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0x66 => {
                instruction_size = 2;
                Instruction { // ROR zeropage
                    operation: Operations::RotateRight,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 5
                }
            },
            0x76 => {
                instruction_size = 2;
                Instruction { // ROR zeropage,X
                    operation: Operations::RotateRight,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 6
                }
            },
            0x6e => {
                instruction_size = 3;
                Instruction { // ROR absolute
                    operation: Operations::RotateRight,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 6
                }
            },
            0x7e => {
                instruction_size = 3;
                Instruction { // ROR absolute,X
                    operation: Operations::RotateRight,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.x),
                    cycle_count: 7
                }
            },
            0x70 => {
                instruction_size = 2;
                Instruction { // BVS relative
                    operation: Operations::BranchOnOverflowSet,
                    addressing: Addressing::RelativeAddress(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0x78 => {
                instruction_size = 1;
                Instruction { // SEI
                    operation: Operations::SetInterruptDisable,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0x85 => {
                instruction_size = 2;
                Instruction { // STA zeropage
                    operation: Operations::StoreAccumulator,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0x95 => {
                instruction_size = 2;
                Instruction { // STA zeropage,X
                    operation: Operations::StoreAccumulator,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0x8d => {
                instruction_size = 3;
                Instruction { // STA absolute
                    operation: Operations::StoreAccumulator,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 4
                }
            },
            0x9d => {
                instruction_size = 3;
                Instruction { // STA absolute,X
                    operation: Operations::StoreAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.x),
                    cycle_count: 5
                }
            },
            0x99 => {
                instruction_size = 3;
                Instruction { // STA absolute,Y
                    operation: Operations::StoreAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.y),
                    cycle_count: 5
                }
            },
            0x81 => {
                instruction_size = 2;
                Instruction { // STA (indirect,X)
                    operation: Operations::StoreAccumulator,
                    addressing: Addressing::PreindexedIndirect(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 6
                }
            },
            0x91 => {
                instruction_size = 2;
                Instruction { // STA (indirect),Y
                    operation: Operations::StoreAccumulator,
                    addressing: Addressing::PostindexedIndirect(self.memory.load(self.pc + 1)?, self.y),
                    cycle_count: 6
                }
            },
            0x84 => {
                instruction_size = 2;
                Instruction { // STY zeropage
                    operation: Operations::StoreY,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0x94 => {
                instruction_size = 2;
                Instruction { // STY zeropage,X
                    operation: Operations::StoreY,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0x8c => {
                instruction_size = 3;
                Instruction { // STY absolute
                    operation: Operations::StoreY,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 4
                }
            },
            0x86 => {
                instruction_size = 2;
                Instruction { // STX zeropage
                    operation: Operations::StoreX,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0x96 => {
                instruction_size = 2;
                Instruction { // STX zeropage,Y
                    operation: Operations::StoreX,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.y),
                    cycle_count: 4
                }
            },
            0x8e => {
                instruction_size = 3;
                Instruction { // STX absolute
                    operation: Operations::StoreX,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 4
                }
            },
            0x88 => {
                instruction_size = 1;
                Instruction { // DEY
                    operation: Operations::DecrementY,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0x8a => {
                instruction_size = 1;
                Instruction { // TXA
                    operation: Operations::TransferXToAccumulator,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0x90 => {
                instruction_size = 2;
                Instruction { // BCC relative
                    operation: Operations::BranchOnCarryClear,
                    addressing: Addressing::RelativeAddress(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0x98 => {
                instruction_size = 1;
                Instruction { // TYA
                    operation: Operations::TransferYToAccumulator,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0x9a => {
                instruction_size = 1;
                Instruction { // TXS
                    operation: Operations::TransferXToStackPointer,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0xa0 => {
                instruction_size = 2;
                Instruction { // LDY immediate
                    operation: Operations::LoadY,
                    addressing: Addressing::Immediate(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0xa4 => {
                instruction_size = 2;
                Instruction { // LDY zeropage
                    operation: Operations::LoadY,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0xb4 => {
                instruction_size = 2;
                Instruction { // LDY zeropage,X
                    operation: Operations::LoadY,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0xac => {
                instruction_size = 3;
                Instruction { // LDY absolute
                    operation: Operations::LoadY,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 4
                }
            },
            0xbc => {
                instruction_size = 3;
                Instruction { // LDY absolute,X
                    operation: Operations::LoadY,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0xa9 => {
                instruction_size = 2;
                Instruction { // LDA immediate
                    operation: Operations::LoadAccumulator,
                    addressing: Addressing::Immediate(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0xa5 => {
                instruction_size = 2;
                Instruction { // LDA zeropage
                    operation: Operations::LoadAccumulator,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0xb5 => {
                instruction_size = 2;
                Instruction { // LDA zeropage,X
                    operation: Operations::LoadAccumulator,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0xad => {
                instruction_size = 3;
                Instruction { // LDA absolute
                    operation: Operations::LoadAccumulator,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 4
                }
            },
            0xbd => {
                instruction_size = 3;
                Instruction { // LDA absolute,X
                    operation: Operations::LoadAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0xb9 => {
                instruction_size = 3;
                Instruction { // LDA absolute,Y
                    operation: Operations::LoadAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.y),
                    cycle_count: 4
                }
            },
            0xa1 => {
                instruction_size = 2;
                Instruction { // LDA (indirect,X)
                    operation: Operations::LoadAccumulator,
                    addressing: Addressing::PreindexedIndirect(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 6
                }
            },
            0xb1 => {
                instruction_size = 2;
                Instruction { // LDA (indirect), Y
                    operation: Operations::LoadAccumulator,
                    addressing: Addressing::PostindexedIndirect(self.memory.load(self.pc + 1)?, self.y),
                    cycle_count: 5
                }
            },
            0xa8 => {
                instruction_size = 1;
                Instruction { // TAY
                    operation: Operations::TransferAccumulatorToY,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0xaa => {
                instruction_size = 1;
                Instruction { // TAX
                    operation: Operations::TransferAccumulatorToX,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0xb0 => {
                instruction_size = 2;
                Instruction { // BCS
                    operation: Operations::BranchOnCarrySet,
                    addressing: Addressing::RelativeAddress(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0xb8 => {
                instruction_size = 1;
                Instruction { // CLV
                    operation: Operations::ClearOverflow,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0xba => {
                instruction_size = 1;
                Instruction { // TSX
                    operation: Operations::TransferStackPointerToX,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0xc0 => {
                instruction_size = 2;
                Instruction { // CPY immediate
                    operation: Operations::CompareWithY,
                    addressing: Addressing::Immediate(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0xc4 => {
                instruction_size = 2;
                Instruction { // CPY zeropage
                    operation: Operations::CompareWithY,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0xcc => {
                instruction_size = 3;
                Instruction { // CPY absolute
                    operation: Operations::CompareWithY,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 4
                }
            },
            0xc9 => {
                instruction_size = 2;
                Instruction { // CMP immediate
                    operation: Operations::CompareWithAccumulator,
                    addressing: Addressing::Immediate(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0xc5 => {
                instruction_size = 2;
                Instruction { // CMP zeropage
                    operation: Operations::CompareWithAccumulator,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0xd5 => {
                instruction_size = 2;
                Instruction { // CMP zeropage,X
                    operation: Operations::CompareWithAccumulator,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0xcd => {
                instruction_size = 3;
                Instruction { // CMP absolute
                    operation: Operations::CompareWithAccumulator,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 4
                }
            },
            0xdd => {
                instruction_size = 3;
                Instruction { // CMP absolute,X
                    operation: Operations::CompareWithAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0xd9 => {
                instruction_size = 3;
                Instruction { // CMP absolute, Y
                    operation: Operations::CompareWithAccumulator,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.y),
                    cycle_count: 4
                }
            },
            0xc1 => {
                instruction_size = 2;
                Instruction { // CMP (indirect,X)
                    operation: Operations::CompareWithAccumulator,
                    addressing: Addressing::PreindexedIndirect(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 6
                }
            },
            0xd1 => {
                instruction_size = 2;
                Instruction { // CMP (indirect),Y
                    operation: Operations::CompareWithAccumulator,
                    addressing: Addressing::PostindexedIndirect(self.memory.load(self.pc + 1)?, self.y),
                    cycle_count: 5
                }
            },
            0xc6 => {
                instruction_size = 2;
                Instruction { // DEC zeropage
                    operation: Operations::DecrementMemory,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 5
                }
            },
            0xd6 => {
                instruction_size = 2;
                Instruction { // DEC zeropage,X
                    operation: Operations::DecrementMemory,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 6
                }
            },
            0xce => {
                instruction_size = 3;
                Instruction { // DEC absolute
                    operation: Operations::DecrementMemory,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 6
                }
            },
            0xde => {
                instruction_size = 3;
                Instruction { // DEC absolute,X
                    operation: Operations::DecrementMemory,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.x),
                    cycle_count: 7
                }
            },
            0xc8 => {
                instruction_size = 1;
                Instruction { // INY
                    operation: Operations::IncrementY,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0xca => {
                instruction_size = 1;
                Instruction { // DEX
                    operation: Operations::DecrementX,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0xd0 => {
                instruction_size = 2;
                Instruction { // BNE
                    operation: Operations::BranchOnNotEqual,
                    addressing: Addressing::RelativeAddress(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0xd8 => {
                instruction_size = 1;
                Instruction { // CLD
                    operation: Operations::ClearDecimal,
                    addressing: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0xe0 => {
                instruction_size = 2;
                Instruction { // CPX immediate
                    operation: Operations::CompareWithX,
                    addressing: Addressing::Immediate(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0xe4 => {
                instruction_size = 2;
                Instruction { // CPX zeropage
                    operation: Operations::CompareWithX,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0xec => {
                instruction_size = 3;
                Instruction { // CPX absolute
                    operation: Operations::CompareWithX,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 4
                }
            },
            0xe9 => {
                instruction_size = 2;
                Instruction { // SBC immediate
                    operation: Operations::SubtractWithCarry,
                    addressing: Addressing::Immediate(self.memory.load(self.pc + 1)?),
                    cycle_count: 2
                }
            },
            0xe5 => {
                instruction_size = 2;
                Instruction { // SBC zeropage
                    operation: Operations::SubtractWithCarry,
                    addressing: Addressing::Zeropage(self.memory.load(self.pc + 1)?),
                    cycle_count: 3
                }
            },
            0xf5 => {
                instruction_size = 2;
                Instruction { // SBC zeropage,X
                    operation: Operations::SubtractWithCarry,
                    addressing: Addressing::IndexedZeropage(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0xed => {
                instruction_size = 3;
                Instruction { // SBC absolute
                    operation: Operations::SubtractWithCarry,
                    addressing: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)?),
                    cycle_count: 4
                }
            },
            0xfd => {
                instruction_size = 3;
                Instruction { // SBC absolute,X
                    operation: Operations::SubtractWithCarry,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.x),
                    cycle_count: 4
                }
            },
            0xf9 => {
                instruction_size = 3;
                Instruction { // SBC absolute,Y
                    operation: Operations::SubtractWithCarry,
                    addressing: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1)?, self.y),
                    cycle_count: 4
                }
            },
            0xe1 => {
                instruction_size = 2;
                Instruction { // SBC (indirect,X)
                    operation: Operations::SubtractWithCarry,
                    addressing: Addressing::PreindexedIndirect(self.memory.load(self.pc + 1)?, self.x),
                    cycle_count: 6
                }
            },
            0xf1 => {
                instruction_size = 2;
                Instruction { // SBC (indirect),Y
                    operation: Operations::SubtractWithCarry,
                    addressing: Addressing::PostindexedIndirect(self.memory.load(self.pc + 1)?, self.y),
                    cycle_count: 5
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
        Ok(instruction)
    }
}
