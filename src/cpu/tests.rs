use super::*;

#[test]
fn initializes() {
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
fn loads_little_endian_word () {
    let mut mem = Memory::new(64*1024).unwrap();

    mem.load_rom(0x0000, &vec![0xab, 0xcd]);
    mem.load_rom(0xfffe, &vec![0xdc, 0xba]);

    let cpu = Cpu::new(mem);
    assert_eq!(0xcdab, cpu.load_little_endian_u16(0x0000).unwrap());
    assert_eq!(0xbadc, cpu.load_little_endian_u16(0xfffe).unwrap());
}

#[test]
#[should_panic]
fn panics_on_invalid_word_read () {
    let mem = Memory::new(64*1024).unwrap();
    let cpu = Cpu::new(mem);
    _ = cpu.load_little_endian_u16(0xffff);
}


#[test]
fn resets_properly() {
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
    assert_eq!(cpu.load_little_endian_u16(0xfffc).unwrap(), 0x0302);
    assert_eq!(cpu.cycles_busy, 1);
}

#[test]
fn can_fetch_brk_ora_instructions() {
    let mut mem = Memory::new(16*1024).unwrap();
    let rom = vec![
        0x00,
        0x09, 0x0a,
        0x05, 0x01,
        0x15, 0x01,
        0x0d, 0x01, 0x02,
        0x1d, 0x01, 0x02,
        0x19, 0x04, 0xa2,
        0x01, 0x03,
        0x11, 0x03
    ];
    mem.load_rom(0x1000, &rom);

    let mut cpu = Cpu::new(mem);

    assert_eq!(Instruction {
        operation: Operations::SoftwareInterrupt,
        addressing: Addressing::Implied,
        cycle_count: 7
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::Immediate(0x0a),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::Zeropage(0x01),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::IndexedZeropage(0x01, 0x00),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::Absolute(0x0201),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::IndexedAbsolute(0x0201, 0x00),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::IndexedAbsolute(0xa204, 0x00),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::PreindexedIndirect(0x03, 0x00),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::PostindexedIndirect(0x03, 0x00),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_asl_instructions() {
    let mut mem = Memory::new(16*1024).unwrap();
    let rom = vec![
        0x0a,
        0x06, 0x0a,
        0x16, 0x01,
        0x0e, 0x10, 0x45,
        0x1e, 0x11, 0x45
    ];
    mem.load_rom(0x1000, &rom);

    let mut cpu = Cpu::new(mem);
    cpu.x = 0x0a;

    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        addressing: Addressing::Zeropage(0x0a),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        addressing: Addressing::IndexedZeropage(0x01, 0x0a),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        addressing: Addressing::Absolute(0x4510),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        addressing: Addressing::IndexedAbsolute(0x4511, 0x0a),
        cycle_count: 7
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_php_instruction () {
    let rom = vec![
        0x08,
        0x0a
    ];
    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    assert_eq!(Instruction {
        operation: Operations::PushStatusRegister,
        addressing: Addressing::Implied,
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_bpl_instruction () {
    let rom = vec![
        0x10, 0x1b,
        0x0a
    ];
    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    assert_eq!(Instruction {
        operation: Operations::BranchOnPlus,
        addressing: Addressing::RelativeAddress(0x1b),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16)
}

#[test]
fn can_fetch_clc_instruction () {
    let rom = vec![
        0x18
    ];
    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    assert_eq!(Instruction {
        operation: Operations::ClearCarry,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_jsr_instruction () {
    let rom = vec![
        0x20, 0xff, 0x10
    ];
    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    assert_eq!(Instruction {
        operation: Operations::JumpSubroutine,
        addressing: Addressing::Absolute(0x10ff),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_and_instructions () {
    let rom = vec![
        0x29, 0x50,
        0x25, 0x34,
        0x35, 0x10,
        0x2d, 0x00, 0x20,
        0x3d, 0x00, 0x20,
        0x39, 0x00, 0x20,
        0x21, 0x15,
        0x31, 0x30
    ];
    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);
    cpu.x = 0xa1;
    cpu.y = 0xa2;

    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::Immediate(0x50),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::Zeropage(0x34),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::IndexedZeropage(0x10, 0xa1),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::Absolute(0x2000),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::IndexedAbsolute(0x2000, 0xa1),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::IndexedAbsolute(0x2000, 0xa2),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::PreindexedIndirect(0x15, 0xa1),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::PostindexedIndirect(0x30, 0xa2),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_bit_instructions() {
    let rom = vec![
        0x24, 0x80,
        0x2c, 0x98, 0x99
    ];
    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    assert_eq!(Instruction {
        operation: Operations::BitTest,
        addressing: Addressing::Zeropage(0x80),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::BitTest,
        addressing: Addressing::Absolute(0x9998),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_rol_instructions() {
    let rom = vec![
        0x2a,
        0x26, 0x55,
        0x36, 0xcc,
        0x2e, 0xaa, 0xcc,
        0x3e, 0xcc, 0xaa
    ];
    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.x = 0x15;

    assert_eq!(Instruction {
        operation: Operations::RotateLeft,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::RotateLeft,
        addressing: Addressing::Zeropage(0x55),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::RotateLeft,
        addressing: Addressing::IndexedZeropage(0xcc, 0x15),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::RotateLeft,
        addressing: Addressing::Absolute(0xccaa),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::RotateLeft,
        addressing: Addressing::IndexedAbsolute(0xaacc, 0x15),
        cycle_count: 7
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_bmi_sec_rti_instructions() {
    let rom = vec![
        0x30, 0x24,
        0x38,
        0x40
    ];
    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    assert_eq!(Instruction {
        operation: Operations::BranchOnMinus,
        addressing: Addressing::RelativeAddress(0x24),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::SetCarry,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ReturnFromInterrupt,
        addressing: Addressing::Implied,
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_eor_instructions() {
    let rom = vec![
        0x49, 0x20,
        0x45, 0x02,
        0x55, 0x08,
        0x4d, 0xfe, 0xff,
        0x5d, 0x02, 0x02,
        0x59, 0x03, 0x00,
        0x41, 0x80,
        0x51, 0x70
    ];

    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.x = 0x05;
    cpu.y = 0x10;

    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::Immediate(0x20),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::Zeropage(0x02),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::IndexedZeropage(0x08, 0x05),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::Absolute(0xfffe),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::IndexedAbsolute(0x0202, 0x05),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::IndexedAbsolute(0x0003, 0x10),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::PreindexedIndirect(0x80, 0x05),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::PostindexedIndirect(0x70, 0x10),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_lsr_instructions() {
    let rom = vec![
        0x4a,
        0x46, 0x07,
        0x56, 0x06,
        0x4e, 0x01, 0x12,
        0x5e, 0x04, 0x12
    ];

    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.x = 0x02;

    assert_eq!(Instruction {
        operation: Operations::LogicalShiftRight,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LogicalShiftRight,
        addressing: Addressing::Zeropage(0x07),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LogicalShiftRight,
        addressing: Addressing::IndexedZeropage(0x06, 0x02),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LogicalShiftRight,
        addressing: Addressing::Absolute(0x1201),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LogicalShiftRight,
        addressing: Addressing::IndexedAbsolute(0x1204, 0x02),
        cycle_count: 7
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_jmp_bvc_cli_rts_instructions() {
    let rom = vec![
        0x4c, 0x13, 0x20,
        0x6c, 0x15, 0x17,
        0x50, 0xfc,
        0x58,
        0x60
    ];

    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    assert_eq!(Instruction {
        operation: Operations::Jump,
        addressing: Addressing::Absolute(0x2013),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::Jump,
        addressing: Addressing::Indirect(0x1715),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::BranchOnOverflowClear,
        addressing: Addressing::RelativeAddress(0xfc),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ClearInterruptDisable,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ReturnFromSubroutine,
        addressing: Addressing::Implied,
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_adc_instructions() {
    let rom = vec![
        0x69, 0x83,
        0x65, 0x10,
        0x75, 0x10,
        0x6d, 0x37, 0xe0,
        0x7d, 0x38, 0xe1,
        0x79, 0x39, 0xe2,
        0x61, 0x00,
        0x71, 0x80
    ];
    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.x = 0x01;
    cpu.y = 0x02;

    assert_eq!(Instruction {
        operation: Operations::AddWithCarry,
        addressing: Addressing::Immediate(0x83),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::AddWithCarry,
        addressing: Addressing::Zeropage(0x10),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::AddWithCarry,
        addressing: Addressing::IndexedZeropage(0x10, 0x01),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::AddWithCarry,
        addressing: Addressing::Absolute(0xe037),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::AddWithCarry,
        addressing: Addressing::IndexedAbsolute(0xe138, 0x01),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::AddWithCarry,
        addressing: Addressing::IndexedAbsolute(0xe239, 0x02),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::AddWithCarry,
        addressing: Addressing::PreindexedIndirect(0x00, 0x01),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::AddWithCarry,
        addressing: Addressing::PostindexedIndirect(0x80, 0x02),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_ror_instructions() {
    let rom = vec![
        0x6a,
        0x66, 0x41,
        0x76, 0x42,
        0x6e, 0x20, 0x0a,
        0x7e, 0x21, 0x0a
    ];
    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.x = 0x10;
    
    assert_eq!(Instruction {
        operation: Operations::RotateRight,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::RotateRight,
        addressing: Addressing::Zeropage(0x41),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::RotateRight,
        addressing: Addressing::IndexedZeropage(0x42, 0x10),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::RotateRight,
        addressing: Addressing::Absolute(0x0a20),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::RotateRight,
        addressing: Addressing::IndexedAbsolute(0x0a21, 0x10),
        cycle_count: 7
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_bvs_sei_sta_instructions() {
    let rom = vec![
        0x70, 0x02,
        0x78,
        0x85, 0x05,
        0x95, 0x10,
        0x8d, 0x00, 0x38,
        0x9d, 0x01, 0x38,
        0x99, 0x08, 0x38,
        0x81, 0x50,
        0x91, 0x53
    ];

    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.x = 0x5;
    cpu.y = 0x6;

    assert_eq!(Instruction {
        operation: Operations::BranchOnOverflowSet,
        addressing: Addressing::RelativeAddress(0x02),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::SetInterruptDisable,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::StoreAccumulator,
        addressing: Addressing::Zeropage(0x05),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::StoreAccumulator,
        addressing: Addressing::IndexedZeropage(0x10, 0x5),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::StoreAccumulator,
        addressing: Addressing::Absolute(0x3800),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::StoreAccumulator,
        addressing: Addressing::IndexedAbsolute(0x3801, 0x05),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::StoreAccumulator,
        addressing: Addressing::IndexedAbsolute(0x3808, 0x06),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::StoreAccumulator,
        addressing: Addressing::PreindexedIndirect(0x50, 0x05),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::StoreAccumulator,
        addressing: Addressing::PostindexedIndirect(0x53, 0x06),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_stx_sty_instructions() {
    let rom = vec![
        0x86, 0x03,
        0x96, 0x04,
        0x8e, 0x05, 0x01,
        0x84, 0x00,
        0x94, 0x01,
        0x8c, 0x06, 0x01
    ];
    let mut mem = Memory::new(64 * 1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.x = 0x8;
    cpu.y = 0x9;

    assert_eq!(Instruction {
        operation: Operations::StoreX,
        addressing: Addressing::Zeropage(0x03),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::StoreX,
        addressing: Addressing::IndexedZeropage(0x04, 0x09),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::StoreX,
        addressing: Addressing::Absolute(0x0105),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::StoreY,
        addressing: Addressing::Zeropage(0x00),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::StoreY,
        addressing: Addressing::IndexedZeropage(0x01, 0x08),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::StoreY,
        addressing: Addressing::Absolute(0x0106),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_dey_txa_bcc_tya_txs_instructions() {
    let rom = vec![
        0x88,
        0x8a,
        0x90, 0xf6,
        0x98,
        0x9a
    ];
    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.x = 0x0a;

    assert_eq!(Instruction {
        operation: Operations::DecrementY,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::TransferXToAccumulator,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::BranchOnCarryClear,
        addressing: Addressing::RelativeAddress(0xf6),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::TransferYToAccumulator,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::TransferXToStackPointer,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_ldy_instructions() {
    let rom = vec![
        0xa0, 0x10,
        0xa4, 0x03,
        0xb4, 0x04,
        0xac, 0x02, 0x30,
        0xbc, 0x03, 0x30
    ];
    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.x = 0x01;

    assert_eq!(Instruction {
        operation: Operations::LoadY,
        addressing: Addressing::Immediate(0x10),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadY,
        addressing: Addressing::Zeropage(0x03),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadY,
        addressing: Addressing::IndexedZeropage(0x04, 0x01),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadY,
        addressing: Addressing::Absolute(0x3002),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadY,
        addressing: Addressing::IndexedAbsolute(0x3003, 0x01),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}