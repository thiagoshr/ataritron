use super::*;

enum CpuFlags {
	Carry = 0,
	Zero = 1,
	InterruptDisable = 2,
	_Decimal = 3,
	_Unused = 4,
	_Bflag = 5,
	Overflow = 6,
	Negative = 7
}

impl Cpu {
	fn get_flag(&self, flag : CpuFlags) -> bool {
		self.sr & (0x1 << (flag as u8)) != 0
	}

	fn set_flag(&mut self, flag : CpuFlags) {
		self.sr |= 0x1 << (flag as u8);
	}

	fn clear_flag(&mut self, flag : CpuFlags) {
		self.sr &= !(0x1 << (flag as u8));
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn can_read_flags() {
		let mem = Memory::new(64*1024).unwrap();
		let mut cpu = Cpu::new(mem);

		cpu.sr = 0b10101010;
		assert_eq!(false, cpu.get_flag(CpuFlags::Carry));
		assert_eq!(true, cpu.get_flag(CpuFlags::Zero));
		assert_eq!(false, cpu.get_flag(CpuFlags::InterruptDisable));
		assert_eq!(false, cpu.get_flag(CpuFlags::Overflow));
		assert_eq!(true, cpu.get_flag(CpuFlags::Negative));
	}

	#[test]
	fn can_set_flags() {
		let mem = Memory::new(64*1024).unwrap();
		let mut cpu = Cpu::new(mem);

		cpu.set_flag(CpuFlags::Carry);
		cpu.set_flag(CpuFlags::Zero);
		cpu.set_flag(CpuFlags::InterruptDisable);
		cpu.set_flag(CpuFlags::Overflow);
		cpu.set_flag(CpuFlags::Negative);

		assert_eq!(0b11000111, cpu.sr);
	}
}