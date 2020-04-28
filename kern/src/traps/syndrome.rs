use aarch64::ESR_EL1;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Fault {
    AddressSize,
    Translation,
    AccessFlag,
    Permission,
    Alignment,
    TlbConflict,
    Other(u8),
}

impl From<u32> for Fault {
    fn from(val: u32) -> Fault {
        use self::Fault::*;

        let value = val & 0b111100;

        match value {
            0b000000 => AddressSize,
            0b000100 => Translation,
            0b001000 => AccessFlag,
            0b001100 => Permission,
            0b100000 => Alignment,
            0b110000 => TlbConflict,
            _ => Other((val & 0b111111) as u8),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Syndrome {
    Unknown,
    WfiWfe,
    SimdFp,
    IllegalExecutionState,
    Svc(u16),
    Hvc(u16),
    Smc(u16),
    MsrMrsSystem,
    InstructionAbort { kind: Fault, level: u8 },
    PCAlignmentFault,
    DataAbort { kind: Fault, level: u8 },
    SpAlignmentFault,
    TrappedFpu,
    SError,
    Breakpoint,
    Step,
    Watchpoint,
    Brk(u16),
    Other(u32),
}

/// Converts a raw syndrome value (ESR) into a `Syndrome` (ref: D1.10.4).
impl From<u32> for Syndrome {
    fn from(esr: u32) -> Syndrome {
        use self::Syndrome::*;
        let ec = esr >> 26;
        let iss = esr & 0xFFFFFF;
        match ec {
            0b000000 => Unknown,
            0b000001 => WfiWfe,
            0b000111 => SimdFp,
            0b001110 => IllegalExecutionState,
            0b010001 => Svc((iss & 0xFFFF) as u16),
            0b010010 => Hvc((iss & 0xFFFF) as u16),
            0b010011 => Smc((iss & 0xFFFF) as u16),
            0b011000 => MsrMrsSystem,
            0b100000 => InstructionAbort { //lower exception level
                kind: iss.into(),
                level: (iss & 0b11) as u8,
            },
            0b100001 => InstructionAbort { //same exception level
                kind: iss.into(),
                level: (iss & 0b11) as u8,
            },
            0b100010 => PCAlignmentFault, 
            0b100100 => DataAbort {
                kind: iss.into(),
                level: (iss & 0b11) as u8,
            },
            0b100101 => DataAbort {
                kind: iss.into(),
                level: (iss & 0b11) as u8,
            },
            0b100110 => SpAlignmentFault, 
            0b101000 => TrappedFpu,
            0b101111 => SError,
            0b110000 => Breakpoint,
            0b110001 => Breakpoint,
            0b111000 => Breakpoint,
            0b110010 => Step,
            0b110011 => Step,
            0b110100 => Watchpoint,
            0b110101 => Watchpoint,
            0b111100 => Brk((iss & 0xFFFF) as u16),
            other => Other(other),
        }
    }
}
