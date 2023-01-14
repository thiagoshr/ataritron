mod instructions;

use crate::memory::{self, Memory};

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
            sp: 0x00ff, // stack: [0x00ff, 0x03ff] 
            pc: 0x1000, // cartridge first address
            a: 0,
            x: 0,
            y: 0,
            sr: 0,
            memory: mem
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_initializes() {
        let mem = Memory::new(16*1024).unwrap();

        let cpu = Cpu::new(mem);
        assert_eq!(cpu.sp, 0x00ff);
        assert_eq!(cpu.pc, 0x1000);
        assert_eq!(cpu.a, 0x0);
        assert_eq!(cpu.x, 0x0);
        assert_eq!(cpu.y, 0x0);
        assert_eq!(cpu.sr, 0x0);
        assert_eq!(cpu.memory.load(0x0000).unwrap(), 0);
    }
}
