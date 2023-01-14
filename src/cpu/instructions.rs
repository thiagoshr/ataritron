enum Addressing {
	Implied,
	Immediate(u8),
	Absolute(u8),
	Zeropage(u8),
	IndexedAbsolute(u16, u8),
	IndexedZeropage(u8, u8),
	Indirect(u16),
	PreindexedIndirect(u8),
	PostindexedIndirect(u8),
	RelativeAddress(u8), // used in conditional branching instructions
}

