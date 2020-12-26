use crate::cpu::Cpu;
use crate::opcode::addressing_mode::AddressMode;
use crate::opcode::Operation;
use std::string::ToString;

/// Each page is 256 bytes.
const PAGE_SIZE: u16 = 0x100;

/// If the src address is on a different page to dest address.
fn is_on_different_pages(src: u16, dest: u16) -> bool {
    src / PAGE_SIZE != dest / PAGE_SIZE
}

pub struct Branch {
    branch_type: BranchType,

    /// Offset to branch to on success.
    offset: i8,
}

/// The type of branch operation.
enum BranchType {
    /// (B)ranch on (c)arry (s)et.
    Bcs,

    /// (B)ranch on (c)arry (c)lear.
    Bcc,

    /// (B)ranch on (n)ot (e)qual.
    Bne,

    /// Branch on (e)qual.
    Beq,

    /// (B)ranch on (mi)nus.
    Bmi,

    /// (B)ranch on (pl)us.
    Bpl,

    /// (B)ranch on o(v)erflow (s)et.
    Bvs,

    /// (B)ranch on o(v)erflow (c)lear.
    Bvc,
}

impl BranchType {
    /// Convert from the opcode to branch type enum.
    pub fn from_opcode(opcode: u8) -> Option<BranchType> {
        match opcode {
            0x10 => Some(BranchType::Bpl),
            0x30 => Some(BranchType::Bmi),
            0x50 => Some(BranchType::Bvc),
            0x70 => Some(BranchType::Bvs),
            0xB0 => Some(BranchType::Bcs),
            0x90 => Some(BranchType::Bcc),
            0xD0 => Some(BranchType::Bne),
            0xF0 => Some(BranchType::Beq),
            _ => None,
        }
    }

    /// Convert from BranchType to opcode.
    pub fn to_opcode(&self) -> u8 {
        match self {
            BranchType::Bpl => 0x10,
            BranchType::Bmi => 0x30,
            BranchType::Bvc => 0x50,
            BranchType::Bvs => 0x70,
            BranchType::Bcs => 0xB0,
            BranchType::Bcc => 0x90,
            BranchType::Bne => 0xD0,
            BranchType::Beq => 0xF0,
        }
    }
}

impl ToString for BranchType {
    fn to_string(&self) -> String {
        match &self {
            BranchType::Bcs => "BCS",
            BranchType::Bcc => "BCC",
            BranchType::Beq => "BEQ",
            BranchType::Bne => "BNE",
            BranchType::Bmi => "BMI",
            BranchType::Bpl => "BPL",
            BranchType::Bvs => "BVS",
            BranchType::Bvc => "BVC",
        }
        .to_string()
    }
}

impl Branch {
    /// Number of bytes in branch operation.
    const BYTE_COUNT: u16 = 2;

    /// Create a new branch from an opcode.
    pub fn new(opcode: u8, cpu: &Cpu) -> Option<Self> {
        let offset = cpu.memory[(cpu.program_counter + 1) as usize] as i8;

        let branch_type = BranchType::from_opcode(opcode)?;

        Some(Branch {
            branch_type,
            offset,
        })
    }

    /// Specalitve computation of branch value.
    fn branch_value(&self, cpu: &Cpu) -> u16 {
        cpu.program_counter + Self::BYTE_COUNT + self.offset as u16
    }
}

impl Operation for Branch {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += Self::BYTE_COUNT;
        cpu.cycles += 2;

        let should_branch = match self.branch_type {
            BranchType::Bcs => cpu.status.carry,
            BranchType::Bcc => !cpu.status.carry,
            BranchType::Beq => cpu.status.zero,
            BranchType::Bne => !cpu.status.zero,
            BranchType::Bmi => cpu.status.negative,
            BranchType::Bpl => !cpu.status.negative,
            BranchType::Bvs => cpu.status.overflow,
            BranchType::Bvc => !cpu.status.overflow,
        };

        if should_branch {
            let addr = AddressMode::Relative {
                offset: self.offset,
            }
            .to_addr(cpu)
            .unwrap();

            let next_instruction_addr = cpu.program_counter;

            cpu.program_counter = addr;

            cpu.cycles += 1;

            if is_on_different_pages(next_instruction_addr, cpu.program_counter) {
                cpu.cycles += 1;
            }
        }
    }

    fn dump(&self, cpu: &Cpu) -> String {
        format!(
            "{:02X} {:02X}     {} ${:04X}   ",
            self.branch_type.to_opcode(),
            self.offset,
            self.branch_type.to_string(),
            self.branch_value(cpu),
        )
    }
}
