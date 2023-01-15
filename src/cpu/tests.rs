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
        operands: Addressing::Implied,
        cycle_count: 3
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        operands: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch());
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
        operands: Addressing::RelativeAddress(0x1b),
        cycle_count: 2
    }, cpu.fetch());
    assert_eq!(Instruction {
        operation: Operations::ArithmeticShiftLeft,
        operands: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch());
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
        operands: Addressing::Implied,
        cycle_count: 2
    }, cpu.fetch());
    assert_eq!(cpu.pc, 0x1001);
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
        operands: Addressing::Absolute(0x10ff),
        cycle_count: 6
    }, cpu.fetch());
    assert_eq!(cpu.pc, 0x1003);
}

