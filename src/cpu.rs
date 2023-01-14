use crate::memory::Memory;

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
	sr : u8
}

