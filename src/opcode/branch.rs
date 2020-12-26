use crate::cpu::Cpu;
use crate::opcode::Operation;

/// Branch if carry flag set.
pub struct Bcs {
    /// Relative value to branch to.
    relative_value: u8,
}

impl Bcs {
    pub const OPCODE: u8 = 0xB0;

    pub fn new(cpu: &Cpu) -> Self {
        let relative_value = cpu.memory[(cpu.program_counter + 1) as usize];

        Bcs { relative_value }
    }
}

impl Operation for Bcs {
    fn execute(&self, cpu: &mut Cpu) {
        // TODO: Move the constant to a associated constant similar to the OPCODE.
        cpu.program_counter += 2;
        cpu.cycles += 2;

        if cpu.status.carry {
            cpu.program_counter += self.relative_value as u16;
            // TODO: Add cycles if it is a new page?
            cpu.cycles += 1;
        }
    }

    fn dump(&self, cpu: &Cpu) -> String {
        format!(
            "{:02X} {:02X}     BCS ${:04X}   ",
            Self::OPCODE,
            self.relative_value,
            cpu.program_counter + self.relative_value as u16 + 2
        )
    }
}

/// Branch if carry flag clear.
pub struct Bcc {
    /// Relative value to branch to.
    relative_value: u8,
}

impl Bcc {
    pub const OPCODE: u8 = 0x90;

    pub fn new(cpu: &Cpu) -> Self {
        let relative_value = cpu.memory[(cpu.program_counter + 1) as usize];

        Bcc { relative_value }
    }
}

impl Operation for Bcc {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += 2;
        cpu.cycles += 2;

        if !cpu.status.carry {
            cpu.program_counter += self.relative_value as u16;
            // TODO: Add cycles if it is a new page?
            cpu.cycles += 1;
        }
    }

    fn dump(&self, cpu: &Cpu) -> String {
        format!(
            "{:02X} {:02X}     BCC ${:04X}   ",
            Self::OPCODE,
            self.relative_value,
            cpu.program_counter + self.relative_value as u16 + 2
        )
    }
}

/// Branch if overflow set.
pub struct Bvs {
    /// Relative value to branch to.
    relative_value: u8,
}

impl Bvs {
    pub const OPCODE: u8 = 0x70;

    pub fn new(cpu: &Cpu) -> Self {
        let relative_value = cpu.memory[(cpu.program_counter + 1) as usize];

        Bvs { relative_value }
    }
}

impl Operation for Bvs {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += 2;
        cpu.cycles += 2;

        if cpu.status.overflow {
            cpu.program_counter += self.relative_value as u16;
            // TODO: Add cycles if it is a new page?
            cpu.cycles += 1;
        }
    }

    fn dump(&self, cpu: &Cpu) -> String {
        format!(
            "{:02X} {:02X}     BVS ${:04X}   ",
            Self::OPCODE,
            self.relative_value,
            cpu.program_counter + self.relative_value as u16 + 2
        )
    }
}

/// Branch if overflow clear.
pub struct Bvc {
    /// Relative value to branch to.
    relative_value: u8,
}

impl Bvc {
    pub const OPCODE: u8 = 0x50;

    pub fn new(cpu: &Cpu) -> Self {
        let relative_value = cpu.memory[(cpu.program_counter + 1) as usize];

        Bvc { relative_value }
    }
}

impl Operation for Bvc {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += 2;
        cpu.cycles += 2;

        if !cpu.status.overflow {
            cpu.program_counter += self.relative_value as u16;
            // TODO: Add cycles if it is a new page?
            cpu.cycles += 1;
        }
    }

    fn dump(&self, cpu: &Cpu) -> String {
        format!(
            "{:02X} {:02X}     BVC ${:04X}   ",
            Self::OPCODE,
            self.relative_value,
            cpu.program_counter + self.relative_value as u16 + 2
        )
    }
}

/// Branch if equal to zero.
pub struct Beq {
    /// Relative value to branch to.
    relative_value: u8,
}

impl Beq {
    pub const OPCODE: u8 = 0xF0;

    pub fn new(cpu: &Cpu) -> Self {
        let relative_value = cpu.memory[(cpu.program_counter + 1) as usize];

        Beq { relative_value }
    }
}

impl Operation for Beq {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += 2;
        cpu.cycles += 2;

        if cpu.status.zero {
            cpu.program_counter += self.relative_value as u16;
            // TODO: Add cycles if it is a new page?
            cpu.cycles += 1;
        }
    }

    fn dump(&self, cpu: &Cpu) -> String {
        format!(
            "{:02X} {:02X}     BEQ ${:04X}   ",
            Self::OPCODE,
            self.relative_value,
            cpu.program_counter + self.relative_value as u16 + 2
        )
    }
}

/// Branch if equal to zero.
pub struct Bne {
    /// Relative value to branch to.
    relative_value: u8,
}

impl Bne {
    pub const OPCODE: u8 = 0xD0;

    pub fn new(cpu: &Cpu) -> Self {
        let relative_value = cpu.memory[(cpu.program_counter + 1) as usize];

        Bne { relative_value }
    }
}

impl Operation for Bne {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += 2;
        cpu.cycles += 2;

        if !cpu.status.zero {
            cpu.program_counter += self.relative_value as u16;
            // TODO: Add cycles if it is a new page?
            cpu.cycles += 1;
        }
    }

    fn dump(&self, cpu: &Cpu) -> String {
        format!(
            "{:02X} {:02X}     BNE ${:04X}   ",
            Self::OPCODE,
            self.relative_value,
            cpu.program_counter + self.relative_value as u16 + 2
        )
    }
}

/// Branch if positive.
pub struct Bpl {
    /// Relative value to branch to.
    relative_value: u8,
}

impl Bpl {
    pub const OPCODE: u8 = 0x10;

    pub fn new(cpu: &Cpu) -> Self {
        let relative_value = cpu.memory[(cpu.program_counter + 1) as usize];

        Bpl { relative_value }
    }
}

impl Operation for Bpl {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += 2;
        cpu.cycles += 2;

        if !cpu.status.negative {
            cpu.program_counter += self.relative_value as u16;
            // TODO: Add cycles if it is a new page?
            cpu.cycles += 1;
        }
    }

    fn dump(&self, cpu: &Cpu) -> String {
        format!(
            "{:02X} {:02X}     BPL ${:04X}   ",
            Self::OPCODE,
            self.relative_value,
            cpu.program_counter + self.relative_value as u16 + 2
        )
    }
}
