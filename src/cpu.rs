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

    /// stack pointer -- 8bits, stack is on page 0x01
    sp: u8,

    /// status register (flags)
    sr: u8,

    memory: Memory,

    cycles_busy: u8
}

impl Cpu {
    fn new(mem: Memory) -> Cpu {
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
                    operands: Addressing::Implied,
                    cycle_count: 7
                }
            },
            0x09 => {
                instruction_size = 2;
                Instruction {  // ORA immediate
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::Immediate(self.memory.load(self.pc + 1).unwrap()),
                    cycle_count: 2
                }
            },
            0x05 => {
                instruction_size = 2;
                Instruction { // ORA zeropage
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::Zeropage(self.memory.load(self.pc + 1).unwrap()),
                    cycle_count: 3
                }
            },
            0x15 => {
                instruction_size = 2;
                Instruction { // ORA indexed zeropage
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::IndexedZeropage(self.memory.load(self.pc + 1).unwrap(), self.x),
                    cycle_count: 4
                }
            },
            0x0d => {
                instruction_size = 3;
                Instruction { // ORA absolute
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)),
                    cycle_count: 4
                }
            },
            0x1d => {
                instruction_size = 3;
                Instruction { // ORA absolute,X
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1), self.x),
                    cycle_count: 4
                }
            },
            0x19 => {
                instruction_size = 3;
                Instruction { // ORA absolute,Y
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1), self.y),
                    cycle_count: 4
                }
            },
            0x01 => {
                instruction_size = 2;
                Instruction { // ORA (indirect,X)
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::PreindexedIndirect(self.memory.load(self.pc + 1).unwrap(), self.x),
                    cycle_count: 6
                }
            },
            0x11 => {
                instruction_size = 2;
                Instruction { // ORA (indirect), Y
                    operation: Operations::InclusiveOrWithAccumulator,
                    operands: Addressing::PostindexedIndirect(self.memory.load(self.pc + 1).unwrap(), self.y),
                    cycle_count: 5
                }
            },
            0x0a => {
                instruction_size = 1;
                Instruction { // ASL accumulator (implied)
                    operation: Operations::ArithmeticShiftLeft,
                    operands: Addressing::Implied,
                    cycle_count: 2
                }
            },
            0x06 => {
                instruction_size = 2;
                Instruction { // ASL zeropage 
                    operation: Operations::ArithmeticShiftLeft,
                    operands: Addressing::Zeropage(self.memory.load(self.pc + 1).unwrap()),
                    cycle_count: 5
                }
            },
            0x16 => {
                instruction_size = 2;
                Instruction { /// ASL zeropage,X
                    operation: Operations::ArithmeticShiftLeft,
                    operands: Addressing::IndexedZeropage(self.memory.load(self.pc + 1).unwrap(), self.x),
                    cycle_count: 6
                }
            },
            0x0e => {
                instruction_size = 3;
                Instruction { // ASL absolute
                    operation: Operations::ArithmeticShiftLeft,
                    operands: Addressing::Absolute(self.load_little_endian_u16(self.pc + 1)),
                    cycle_count: 6
                }
            },
            0x1e => {
                instruction_size = 3;
                Instruction { // ASL absolute, X
                    operation: Operations::ArithmeticShiftLeft,
                    operands: Addressing::IndexedAbsolute(self.load_little_endian_u16(self.pc + 1), self.x),
                    cycle_count: 7
                }
            }
            _ => {
                instruction_size = 1;
                Instruction {
                    operation: Operations::NoOperation,
                    operands: Addressing::Implied,
                    cycle_count: 2
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
        assert_eq!(cpu.sp, 0xff);
        assert_eq!(cpu.pc, 0x1000);
        assert_eq!(cpu.a, 0x0);
        assert_eq!(cpu.x, 0x0);
        assert_eq!(cpu.y, 0x0);
        assert_eq!(cpu.sr, 0x0);
        assert_eq!(cpu.memory.load(0x0000).unwrap(), 0);
        assert_eq!(cpu.cycles_busy, 0);
    }

    #[test]
    fn cpu_loads_little_endian_word () {
        let mut mem = Memory::new(64*1024).unwrap();

        mem.load_rom(0x0000, &vec![0xab, 0xcd]);
        mem.load_rom(0xfffe, &vec![0xdc, 0xba]);

        let cpu = Cpu::new(mem);
        assert_eq!(0xcdab, cpu.load_little_endian_u16(0x0000));
        assert_eq!(0xbadc, cpu.load_little_endian_u16(0xfffe));
    }

    #[test]
    #[should_panic]
    fn cpu_panics_on_invalid_word_read () {
        let mem = Memory::new(64*1024).unwrap();
        let cpu = Cpu::new(mem);
        _ = cpu.load_little_endian_u16(0xffff);
    }


    #[test]
    fn cpu_resets_properly() {
        let mut mem = Memory::new(65536).unwrap();

        mem.load_rom(0xfffc, &vec![0x02, 0x03]);
        let mut cpu = Cpu::new(mem);
        cpu.reset();
        assert_eq!(cpu.sp, 0xff);
        assert_eq!(cpu.pc, 0x0302);
        assert_eq!(cpu.a, 0x0);
        assert_eq!(cpu.x, 0x0);
        assert_eq!(cpu.y, 0x0);
        assert_eq!(cpu.sr, 0x0);
        assert_eq!(cpu.load_little_endian_u16(0xfffc), 0x0302);
        assert_eq!(cpu.cycles_busy, 1);
    }

    #[test]
    fn can_fetch_brk_ora_instructions() {
        let mut mem = Memory::new(16*1024).unwrap();
        mem.load_rom(0x1000, &vec![
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
            operands: Addressing::Implied,
            cycle_count: 7
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::Immediate(0x0a),
            cycle_count: 2
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::Zeropage(0x01),
            cycle_count: 3
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::IndexedZeropage(0x01, 0x00),
            cycle_count: 4
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::Absolute(0x0201),
            cycle_count: 4
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::IndexedAbsolute(0x0201, 0x00),
            cycle_count: 4
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::IndexedAbsolute(0xa204, 0x00),
            cycle_count: 4
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::PreindexedIndirect(0x03, 0x00),
            cycle_count: 6
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::InclusiveOrWithAccumulator,
            operands: Addressing::PostindexedIndirect(0x03, 0x00),
            cycle_count: 5
        }, cpu.fetch());
    }

    #[test]
    fn can_fetch_asl_instructions() {
        let mut mem = Memory::new(16*1024).unwrap();
        mem.load_rom(0x1000, &vec![
            0x0a,
            0x06, 0x0a,
            0x16, 0x01,
            0x0e, 0x10, 0x45,
            0x1e, 0x11, 0x45
        ]);

        let mut cpu = Cpu::new(mem);
        cpu.x = 0x0a;

        assert_eq!(Instruction {
            operation: Operations::ArithmeticShiftLeft,
            operands: Addressing::Implied,
            cycle_count: 2
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::ArithmeticShiftLeft,
            operands: Addressing::Zeropage(0x0a),
            cycle_count: 5
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::ArithmeticShiftLeft,
            operands: Addressing::IndexedZeropage(0x01, 0x0a),
            cycle_count: 6
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::ArithmeticShiftLeft,
            operands: Addressing::Absolute(0x4510),
            cycle_count: 6
        }, cpu.fetch());
        assert_eq!(Instruction {
            operation: Operations::ArithmeticShiftLeft,
            operands: Addressing::IndexedAbsolute(0x4511, 0x0a),
            cycle_count: 7
        }, cpu.fetch());
    }
}
