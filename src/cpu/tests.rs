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
    assert_eq!(0xcdab, cpu.load_little_endian_u16(0x0000));
    assert_eq!(0xbadc, cpu.load_little_endian_u16(0xfffe));
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
    assert_eq!(cpu.load_little_endian_u16(0xfffc), 0x0302);
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
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::Immediate(0x0a),
        cycle_count: 2
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::Zeropage(0x01),
        cycle_count: 3
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::IndexedZeropage(0x01, 0x00),
        cycle_count: 4
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::Absolute(0x0201),
        cycle_count: 4
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::IndexedAbsolute(0x0201, 0x00),
        cycle_count: 4
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::IndexedAbsolute(0xa204, 0x00),
        cycle_count: 4
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::PreindexedIndirect(0x03, 0x00),
        cycle_count: 6
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::InclusiveOrWithAccumulator,
        addressing: Addressing::PostindexedIndirect(0x03, 0x00),
        cycle_count: 5
    }, cpu.fetch());
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
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        addressing: Addressing::Zeropage(0x0a),
        cycle_count: 5
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        addressing: Addressing::IndexedZeropage(0x01, 0x0a),
        cycle_count: 6
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        addressing: Addressing::Absolute(0x4510),
        cycle_count: 6
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        addressing: Addressing::IndexedAbsolute(0x4511, 0x0a),
        cycle_count: 7
    }, cpu.fetch());
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
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch());
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
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch());
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
    }, cpu.fetch());
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
    }, cpu.fetch());
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
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::Zeropage(0x34),
        cycle_count: 3
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::IndexedZeropage(0x10, 0xa1),
        cycle_count: 4
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::Absolute(0x2000),
        cycle_count: 4
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::IndexedAbsolute(0x2000, 0xa1),
        cycle_count: 4
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::IndexedAbsolute(0x2000, 0xa2),
        cycle_count: 4
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::PreindexedIndirect(0x15, 0xa1),
        cycle_count: 6
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::AndWithAccumulator,
        addressing: Addressing::PostindexedIndirect(0x30, 0xa2),
        cycle_count: 5
    }, cpu.fetch());
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
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::BitTest,
        addressing: Addressing::Absolute(0x9998),
        cycle_count: 4
    }, cpu.fetch());
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
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::RotateLeft,
        addressing: Addressing::Zeropage(0x55),
        cycle_count: 5
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::RotateLeft,
        addressing: Addressing::IndexedZeropage(0xcc, 0x15),
        cycle_count: 6
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::RotateLeft,
        addressing: Addressing::Absolute(0xccaa),
        cycle_count: 6
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::RotateLeft,
        addressing: Addressing::IndexedAbsolute(0xaacc, 0x15),
        cycle_count: 7
    }, cpu.fetch());
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
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::SetCarry,
        addressing: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ReturnFromInterrupt,
        addressing: Addressing::Implied,
        cycle_count: 6
    }, cpu.fetch());
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
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::Zeropage(0x02),
        cycle_count: 3
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::IndexedZeropage(0x08, 0x05),
        cycle_count: 4
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::Absolute(0xfffe),
        cycle_count: 4
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::IndexedAbsolute(0x0202, 0x05),
        cycle_count: 4
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::IndexedAbsolute(0x0003, 0x10),
        cycle_count: 4
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::PreindexedIndirect(0x80, 0x05),
        cycle_count: 6
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ExclusiveOrWithAccumulator,
        addressing: Addressing::PostindexedIndirect(0x70, 0x10),
        cycle_count: 5
    }, cpu.fetch());
    assert_eq!(cpu.pc, 0x1000 + rom.len() as u16);
}