use crate::cpu::Cpu;
use crate::opcode::Operation;
use std::string::ToString;

/// Flag type.
pub enum Flag {
    /// (Cl)ear (C)arry
    Clc,

    /// (Se)t (Carry)
    Sec,

    /// (Cl)ear (I)nterrupt.
    Cli,

    /// (Se)t (I)nterrupt.
    Sei,

    /// (CL)ear o(V)erflow.
    Clv,

    /// (CL)ear (D)ecimal.
    Cld,

    /// (SE)t (D)ecimal,
    Sed,
}

impl Flag {
    const BYTE_COUNT: u16 = 1;
    const CYCLE_LENGTH: u64 = 2;

    /// Convert from the opcode to flag type enum.
    pub fn new(opcode: u8) -> Option<Flag> {
        match opcode {
            0x18 => Some(Flag::Clc),
            0x38 => Some(Flag::Sec),
            0x58 => Some(Flag::Cli),
            0x78 => Some(Flag::Sei),
            0xB8 => Some(Flag::Clv),
            0xD8 => Some(Flag::Cld),
            0xF8 => Some(Flag::Sed),
            _ => None,
        }
    }

    /// Convert from Flag to opcode.
    pub fn to_opcode(&self) -> u8 {
        match &self {
            Flag::Clc => 0x18,
            Flag::Sec => 0x38,
            Flag::Cli => 0x58,
            Flag::Sei => 0x78,
            Flag::Clv => 0xB8,
            Flag::Cld => 0xD8,
            Flag::Sed => 0xF8,
        }
    }
}

impl ToString for Flag {
    fn to_string(&self) -> String {
        match &self {
            Flag::Clc => "CLC",
            Flag::Sec => "SEC",
            Flag::Cli => "CLI",
            Flag::Sei => "SEI",
            Flag::Clv => "CLV",
            Flag::Cld => "CLD",
            Flag::Sed => "SED",
        }
        .to_string()
    }
}

impl Operation for Flag {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += Self::BYTE_COUNT;
        cpu.cycles += Self::CYCLE_LENGTH;

        match &self {
            Flag::Clc => cpu.status.carry = false,
            Flag::Sec => cpu.status.carry = true,
            Flag::Cli => cpu.status.interrupt_disable = false,
            Flag::Sei => cpu.status.interrupt_disable = true,
            Flag::Clv => cpu.status.overflow = false,
            Flag::Cld => cpu.status.decimal = false,
            Flag::Sed => cpu.status.decimal = true,
        }
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        format!(
            "{:02X}        {}        ",
            self.to_opcode(),
            self.to_string(),
        )
    }
}
