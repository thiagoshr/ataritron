mod instructions;

use crate::memory::{self, Memory};
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

    /// stack pointer
    sp: u16,

    /// status register (flags)
    sr: u8,

    memory: Memory,
}

impl Cpu {
    fn new(mem: Memory) -> Cpu {
        Cpu {
            sp: 0x01ff, // stack: [0x0100, 0x01ff] 
            pc: 0x1000, // cartridge first address
            a: 0,
            x: 0,
            y: 0,
            sr: 0,
            memory: mem
        }
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
                    operands: Addressing::Implied
                }
            },
            0x09 => {
                instruction_size = 2;
                Instruction {  // ORA immediate
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::Immediate(self.memory.load(self.pc + 1).unwrap())
                }
            },
            0x05 => {
                instruction_size = 2;
                Instruction { // ORA zeropage
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::Zeropage(self.memory.load(self.pc + 1).unwrap())
                }
            },
            0x15 => {
                instruction_size = 2;
                Instruction { // ORA indexed zeropage
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::IndexedZeropage(self.memory.load(self.pc + 1).unwrap(), self.x)
                }
            },
            0x0d => {
                instruction_size = 3;
                Instruction { // ORA absolute
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1))
                }
            },
            0x1d => {
                instruction_size = 3;
                Instruction { // ORA absolute,X
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1), self.x)
                }
            },
            0x19 => {
                instruction_size = 3;
                Instruction { // ORA absolute,Y
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1), self.y)
                }
            },
            0x01 => {
                instruction_size = 2;
                Instruction { // ORA (indirect,X)
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::PreindexedIndirect(self.memory.load(self.pc + 1).unwrap(), self.x)
                }
            },
            0x11 => {
                instruction_size = 2;
                Instruction { /// ORA (indirect), Y
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::PostindexedIndirect(self.memory.load(self.pc + 1).unwrap(), self.y)
                }
            },
            _ => {
                instruction_size = 1;
                Instruction {
                    operation: Operations::NoOperation,
                    operands: Addressing::Implied
                }
            }
        };

        self.pc += instruction_size;
        instruction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_initializes() {
        let mem = Memory::new(16*1024).unwrap();

        let cpu = Cpu::new(mem);
        assert_eq!(cpu.sp, 0x01ff);
        assert_eq!(cpu.pc, 0x1000);
        assert_eq!(cpu.a, 0x0);
        assert_eq!(cpu.x, 0x0);
        assert_eq!(cpu.y, 0x0);
        assert_eq!(cpu.sr, 0x0);
        assert_eq!(cpu.memory.load(0x0000).unwrap(), 0);
    }

    #[test]
    fn can_fetch_brk_ora_instructions() {
        let mut mem = Memory::new(16*1024).unwrap();
        mem.load_rom(0x1000, vec![
            0x00,
            0x09, 0x0a,
            0x05, 0x01,
            0x15, 0x01,
            0x0d, 0x01, 0x02,
            0x1d, 0x01, 0x02,
            0x19, 0x04, 0xa2,
            0x01, 0x03,
            0x11, 0x03
        ]);

        let mut cpu = Cpu::new(mem);

        assert_eq!(Instruction {
            operation: Operations::SoftwareInterrupt,
            operands: Addressing::Implied
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::Immediate(0x0a)
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::Zeropage(0x01)
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::IndexedZeropage(0x01, 0x00)
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::Absolute(0x0201)
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::IndexedAbsolute(0x0201, 0x00)
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::IndexedAbsolute(0xa204, 0x00)
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::PreindexedIndirect(0x03, 0x00)
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::PostindexedIndirect(0x03, 0x00)
        }, cpu.fetch());
    }
}
