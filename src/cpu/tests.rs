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

#[test]
fn can_fetch_lda_instructions() {
    let rom = vec![
        0xa9, 0xba,
        0xa5, 0x11,
        0xb5, 0x23,
        0xad, 0x50, 0x23,
        0xbd, 0x04, 0x23,
        0xb9, 0x77, 0x42,
        0xa1, 0x80,
        0xb1, 0x33
    ];
    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.x = 5;
    cpu.y = 7;

    assert_eq!(Instruction {
        operation: Operations::LoadAccumulator,
        addressing: Addressing::Immediate(0xba),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadAccumulator,
        addressing: Addressing::Zeropage(0x11),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadAccumulator,
        addressing: Addressing::IndexedZeropage(0x23, 5),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadAccumulator,
        addressing: Addressing::Absolute(0x2350),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadAccumulator,
        addressing: Addressing::IndexedAbsolute(0x2304, 5),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadAccumulator,
        addressing: Addressing::IndexedAbsolute(0x4277, 7),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadAccumulator,
        addressing: Addressing::PreindexedIndirect(0x80, 5),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadAccumulator,
        addressing: Addressing::PostindexedIndirect(0x33, 7),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_tay_tax_bcs_clv_tsx_instructions() {
    let rom = vec![
        0xa8,
        0xaa,
        0xb0, 0xf3,
        0xb8,
        0xba
    ];

    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    assert_eq!(Instruction {
        operation: Operations::TransferAccumulatorToY,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::TransferAccumulatorToX,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::BranchOnCarrySet,
        addressing: Addressing::RelativeAddress(0xf3),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ClearOverflow,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::TransferStackPointerToX,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_cpy_instructions() {
    let rom = vec![
        0xc0, 0x0a,
        0xc4, 0xf0,
        0xcc, 0x08, 0x50
    ];

    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    assert_eq!(Instruction {
        operation: Operations::CompareWithY,
        addressing: Addressing::Immediate(0x0a),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::CompareWithY,
        addressing: Addressing::Zeropage(0xf0),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::CompareWithY,
        addressing: Addressing::Absolute(0x5008),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_cmp_instructions() {
    let rom = vec![
        0xc9, 0x00,
        0xc5, 0x01,
        0xd5, 0x02,
        0xcd, 0x03, 0x01,
        0xdd, 0x04, 0x02,
        0xd9, 0x05, 0x03,
        0xc1, 0x06,
        0xd1, 0x07
    ];

    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.x = 0x08;
    cpu.y = 0x09;

    assert_eq!(Instruction {
        operation: Operations::CompareWithAccumulator,
        addressing: Addressing::Immediate(0x00),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::CompareWithAccumulator,
        addressing: Addressing::Zeropage(0x01),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::CompareWithAccumulator,
        addressing: Addressing::IndexedZeropage(0x02, 0x08),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::CompareWithAccumulator,
        addressing: Addressing::Absolute(0x0103),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::CompareWithAccumulator,
        addressing: Addressing::IndexedAbsolute(0x0204, 0x08),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::CompareWithAccumulator,
        addressing: Addressing::IndexedAbsolute(0x0305, 0x09),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::CompareWithAccumulator,
        addressing: Addressing::PreindexedIndirect(0x06, 0x08),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::CompareWithAccumulator,
        addressing: Addressing::PostindexedIndirect(0x07, 0x09),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_dec_instructions() {
    let rom = vec![
        0xc6, 0x10,
        0xd6, 0x11,
        0xce, 0x12, 0x03,
        0xde, 0x13, 0x03
    ];

    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.x = 0x15;

    assert_eq!(Instruction {
        operation: Operations::DecrementMemory,
        addressing: Addressing::Zeropage(0x10),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::DecrementMemory,
        addressing: Addressing::IndexedZeropage(0x11, 0x15),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::DecrementMemory,
        addressing: Addressing::Absolute(0x0312),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::DecrementMemory,
        addressing: Addressing::IndexedAbsolute(0x0313, 0x15),
        cycle_count: 7
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_iny_dex_bne_cld_cpx_instructions() {
    let rom = vec![
        0xc8,
        0xca,
        0xd0, 0xf6,
        0xd8,
        0xe0, 0x0a,
        0xe4, 0x0b,
        0xec, 0x0c, 0x03
    ];

    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    assert_eq!(Instruction {
        operation: Operations::IncrementY,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::DecrementX,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::BranchOnNotEqual,
        addressing: Addressing::RelativeAddress(0xf6),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::ClearDecimal,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::CompareWithX,
        addressing: Addressing::Immediate(0x0a),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::CompareWithX,
        addressing: Addressing::Zeropage(0x0b),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::CompareWithX,
        addressing: Addressing::Absolute(0x030c),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_sbc_instructions() {
    let rom = vec![
        0xe9, 0x30,
        0xe5, 0x01,
        0xf5, 0x02,
        0xed, 0x02, 0x10,
        0xfd, 0x03, 0x10,
        0xf9, 0x04, 0x10,
        0xe1, 0x03,
        0xf1, 0x04
    ];

    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.x = 0xa1;
    cpu.y = 0xa2;

    assert_eq!(Instruction {
        operation: Operations::SubtractWithCarry,
        addressing: Addressing::Immediate(0x30),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::SubtractWithCarry,
        addressing: Addressing::Zeropage(0x01),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::SubtractWithCarry,
        addressing: Addressing::IndexedZeropage(0x02, 0xa1),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::SubtractWithCarry,
        addressing: Addressing::Absolute(0x1002),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::SubtractWithCarry,
        addressing: Addressing::IndexedAbsolute(0x1003, 0xa1),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::SubtractWithCarry,
        addressing: Addressing::IndexedAbsolute(0x1004, 0xa2),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::SubtractWithCarry,
        addressing: Addressing::PreindexedIndirect(0x03, 0xa1),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::SubtractWithCarry,
        addressing: Addressing::PostindexedIndirect(0x04, 0xa2),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_inc_instructions() {
    let rom = vec![
        0xe6, 0x01,
        0xf6, 0x02,
        0xee, 0x03, 0xfe,
        0xfe, 0x04, 0xfe
    ];

    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.x = 0x05;

    assert_eq!(Instruction {
        operation: Operations::IncrementMemory,
        addressing: Addressing::Zeropage(0x01),
        cycle_count: 5
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::IncrementMemory,
        addressing: Addressing::IndexedZeropage(0x02, 0x05),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::IncrementMemory,
        addressing: Addressing::Absolute(0xfe03),
        cycle_count: 6
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::IncrementMemory,
        addressing: Addressing::IndexedAbsolute(0xfe04, 0x05),
        cycle_count: 7
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_inx_beq_sed_instructions() {
    let rom = vec![
        0xe8,
        0xf0, 0x0a,
        0xf8
    ];

    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    assert_eq!(Instruction {
        operation: Operations::IncrementX,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::BranchOnEqual,
        addressing: Addressing::RelativeAddress(0x0a),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::SetDecimal,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}

#[test]
fn can_fetch_ldx_instructions() {
    let rom = vec![
        0xa2, 0x06,
        0xa6, 0x07,
        0xb6, 0x08,
        0xae, 0x09, 0x16,
        0xbe, 0x0a, 0x16
    ];

    let mut mem = Memory::new(64*1024).unwrap();
    mem.load_rom(0x1000, &rom);
    let mut cpu = Cpu::new(mem);

    cpu.y = 0x0b;

    assert_eq!(Instruction {
        operation: Operations::LoadX,
        addressing: Addressing::Immediate(0x06),
        cycle_count: 2
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadX,
        addressing: Addressing::Zeropage(0x07),
        cycle_count: 3
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadX,
        addressing: Addressing::IndexedZeropage(0x08, 0x0b),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadX,
        addressing: Addressing::Absolute(0x1609),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(Instruction {
        operation: Operations::LoadX,
        addressing: Addressing::IndexedAbsolute(0x160a, 0x0b),
        cycle_count: 4
    }, cpu.fetch().unwrap());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}
