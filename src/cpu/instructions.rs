#[derive(Debug)]
pub enum Addressing {
    Implied,
    Immediate(u8),
    Absolute(u16),
    Zeropage(u8),
    IndexedAbsolute(u16, u8),
    /// (index, X)
    IndexedZeropage(u8, u8),
    Indirect(u16),
    PreindexedIndirect(u8, u8),
    PostindexedIndirect(u8, u8),
    RelativeAddress(u8), // used in conditional branching instructions
}
impl PartialEq for Addressing {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Addressing::Implied, Addressing::Implied) => true,
            (Addressing::Immediate(a), Addressing::Immediate(b)) => a == b,
            (Addressing::Absolute(a), Addressing::Absolute(b)) => a == b,
            (Addressing::Zeropage(a), Addressing::Zeropage(b)) => a == b,
            (Addressing::IndexedAbsolute(a1, a2), Addressing::IndexedAbsolute(b1, b2)) => a1 == b1 && a2 == b2,
            (Addressing::IndexedZeropage(a1, a2), Addressing::IndexedZeropage(b1, b2)) => a1 == b1 && a2 == b2,
            (Addressing::Indirect(a), Addressing::Indirect(b)) => a == b,
            (Addressing::PreindexedIndirect(a1, a2), Addressing::PreindexedIndirect(b1, b2)) => a1 == b1 && a2 == b2,
            (Addressing::PostindexedIndirect(a1, a2), Addressing::PostindexedIndirect(b1, b2)) => a1 == b1 && a2 == b2,
            (Addressing::RelativeAddress(a), Addressing::RelativeAddress(b)) => a == b,
            _ => false
        }
    }
}

#[derive(Debug)]
pub enum Operations {
    LoadAccumulator,
    LoadX,
    LoadY,
    StoreAccumulator,
    StoreX,
    StoreY,
    TransferAccumulatorToX,
    TransferAccumulatorToY,
    TransferStackPointerToX,
    TransferXToAccumulator,
    TransferYToAccumulator,
    PushAccumulator,
    PushStatusRegister,
    PullAccumulator,
    PullStatusRegister,
    DecrementMemory,
    DecrementX,
    DecrementY,
    IncrementMemory,
    IncrementX,
    IncrementY,
    AddWithCarry,
    SubtractWithCarry,
    AndWithAccumulator,
    ExclusiveOrWithAccumulator,
    InclusiveOrWithAccumulator,
    ArithmeticShiftLeft,
    LogicalShiftLeft,
    RotateLeft,
    RotateRight,
    ClearCarry,
    ClearDecimal,
    ClearInterruptDisable,
    ClearOverflow,
    SetCarry,
    SetDecimal,
    SetInterruptDisable,
    CompareWithAccumulator,
    CompareWithX,
    CompareWithY,
    BranchOnCarryClear,
    BranchOnCarrySet,
    BranchOnEqual,
    BranchOnMinus,
    BranchOnNotEqual,
    BranchOnPlus,
    BranchOnOverflowClear,
    BranchOnOverflowSet,
    Jump,
    JumpSubroutine,
    ReturnFromSubroutine,
    SoftwareInterrupt,
    ReturnFromInterrupt,
    BitTest,
    NoOperation
}
impl PartialEq for Operations {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Debug)]
pub struct Instruction {
    pub operation : Operations,
    pub operands : Addressing
}
impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        self.operation == other.operation && self.operands == other.operands
    }
}