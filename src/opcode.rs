/// This file contains the OpCodes and their implementations.
use crate::cpu::Cpu;

pub trait Operation {
    /// Execute the opcode.
    fn execute(&self, cpu: &mut Cpu);

    /// Dump the opcode with the read values.
    fn dump(&self, cpu: &Cpu) -> String;
}

/// Read in the next opcode and set up PC.
pub fn next(cpu: &Cpu) -> Box<dyn Operation> {
    let pc = cpu.program_counter as usize;
    let opcode_raw = cpu.memory[pc];

    match opcode_raw {
        Nop::OPCODE => Box::new(Nop::new()),
        0x4C | 0x6C => Box::new(Jmp::new(opcode_raw, cpu)),
        Jsr::OPCODE => Box::new(Jsr::new(cpu)),
        0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => Box::new(Ldx::new(opcode_raw, cpu)),
        0x86 | 0x96 | 0x8E => Box::new(Stx::new(opcode_raw, cpu)),
        Sec::OPCODE => Box::new(Sec::new()),
        Clc::OPCODE => Box::new(Clc::new()),
        Bcs::OPCODE => Box::new(Bcs::new(cpu)),
        Bcc::OPCODE => Box::new(Bcc::new(cpu)),
        _ => panic!("Unsupported {:X}", opcode_raw),
    }
}

enum AddressingMode {
    ZeroPage,
    Absolute,
    Indirect,
    Immediate,
}

impl AddressingMode {
    /// Number of bytes an operation takes including opcode.
    /// http://obelisk.me.uk/6502/reference.html
    fn get_bytes(&self) -> usize {
        match &self {
            AddressingMode::Absolute => 3,
            AddressingMode::Indirect => 3,
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 2,
        }
    }
}

/// Jmp OpCode.
struct Jmp {
    opcode_raw: u8,

    /// Only Absolute and Indirect are valid modes.
    mode: AddressingMode,

    /// Number of cycles this operation takes.
    cycles: u8,

    /// Address to jump to.
    addr: u16,
}

impl Operation for Jmp {
    /// JMP simply moves to the address.
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter = self.addr;
        cpu.cycles += u64::from(self.cycles);
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        let addr_string = match self.mode {
            AddressingMode::Absolute => format!("${:X}", self.addr),
            AddressingMode::Indirect => panic!("Unsupproted!"),
            _ => panic!("Unexpected!"),
        };

        format!(
            "{:02X} {:02X} {:02X}  JMP  {}",
            self.opcode_raw,
            addr_to_bytes(self.addr).0,
            addr_to_bytes(self.addr).1,
            addr_string
        )
    }
}
/// Assuming little endian, merge the two u8 values into a u16 value.
/// For some reason, the orders are reversed?
/// TODO: Figure out Why?
fn bytes_to_addr(first: u8, second: u8) -> u16 {
    ((second as u16) << 8) | first as u16
}

fn addr_to_bytes(addr: u16) -> (u8, u8) {
    (addr as u8, (addr >> 8) as u8)
}

impl Jmp {
    pub fn new(opcode_raw: u8, cpu: &Cpu) -> Self {
        let pc = cpu.program_counter as usize;
        let addr = bytes_to_addr(cpu.memory[pc + 1], cpu.memory[pc + 2]);
        match opcode_raw {
            // Absolute
            0x4C => Jmp {
                opcode_raw,
                mode: AddressingMode::Absolute,
                cycles: 3,
                addr,
            },
            // Indirect
            0x6C => Jmp {
                opcode_raw,
                mode: AddressingMode::Indirect,
                cycles: 5,
                addr,
            },
            _ => panic!("Unexpected opcode"),
        }
    }
}

struct Ldx {
    opcode_raw: u8,

    /// Only Absolute and Indirect are valid modes.
    mode: AddressingMode,

    /// Number of cycles this operation takes.
    cycles: u8,

    /// Value to load.
    value: u8,
}

/// TODO: Generalize addressing mode to a separate module.
impl Ldx {
    pub fn new(opcode_raw: u8, cpu: &Cpu) -> Self {
        let pc = cpu.program_counter as usize;
        let value = cpu.memory[pc + 1];
        match opcode_raw {
            // Immediate.
            0xA2 => Ldx {
                opcode_raw,
                mode: AddressingMode::Immediate,
                cycles: 2,
                value,
            },
            _ => panic!("Unsupported {}", opcode_raw),
        }
    }
}

impl Operation for Ldx {
    /// JMP simply moves to the address.
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += self.mode.get_bytes() as u16;
        cpu.cycles += u64::from(self.cycles);
        cpu.x = self.value;

        cpu.status.update(cpu.x);
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        let addr_string = match self.mode {
            AddressingMode::Immediate => format!("#${:02X}", self.value),
            AddressingMode::Absolute => format!("${:02X}", self.value),
            _ => panic!("Unsupproted!"),
        };

        format!(
            "{:02X} {:02X}     LDX  {}",
            self.opcode_raw, self.value, addr_string
        )
    }
}

struct Stx {
    opcode_raw: u8,

    /// Only Absolute and Indirect are valid modes.
    mode: AddressingMode,

    /// Number of cycles this operation takes.
    cycles: u8,

    /// Address to store value in register x.
    zero_page_addr: u8,
}

impl Stx {
    pub fn new(opcode_raw: u8, cpu: &Cpu) -> Self {
        let pc = cpu.program_counter as usize;
        let zero_page_addr = cpu.memory[pc + 1];
        match opcode_raw {
            // Immediate.
            0x86 => Stx {
                opcode_raw,
                mode: AddressingMode::ZeroPage,
                cycles: 3,
                zero_page_addr,
            },
            _ => panic!("Unsupported {}", opcode_raw),
        }
    }
}

impl Operation for Stx {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += self.mode.get_bytes() as u16;
        cpu.memory[self.zero_page_addr as usize] = cpu.x;
        cpu.cycles += u64::from(self.cycles);
    }

    fn dump(&self, cpu: &Cpu) -> String {
        let addr_string = match self.mode {
            AddressingMode::ZeroPage => format!("${:02X} = {:02X}", self.zero_page_addr, cpu.x),
            _ => panic!("Unsupported!"),
        };

        format!(
            "{:02X} {:02X}     STX  {}",
            self.opcode_raw, self.zero_page_addr, addr_string
        )
    }
}

struct Jsr {
    /// Address to jump to.
    addr: u16,
}

impl Jsr {
    const OPCODE: u8 = 0x20;

    pub fn new(cpu: &Cpu) -> Self {
        let pc = cpu.program_counter as usize;
        let addr = bytes_to_addr(cpu.memory[pc + 1], cpu.memory[pc + 2]);
        Jsr { addr }
    }
}

impl Operation for Jsr {
    fn execute(&self, cpu: &mut Cpu) {
        // Jsr always 3 bytes.
        let return_address = cpu.program_counter + 3;

        let (pcl, pch) = addr_to_bytes(return_address);

        // Push onto the stack the return address - 1.
        cpu.stack.push(&mut cpu.memory, &[pch, pcl]);

        cpu.program_counter = self.addr;

        // Always 6 cycles for a JSR
        cpu.cycles += 6;
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        let addr_string = format!("${:X}", self.addr);

        format!(
            "{:02X} {:02X} {:02X}  JSR  {}",
            Self::OPCODE,
            addr_to_bytes(self.addr).0,
            addr_to_bytes(self.addr).1,
            addr_string
        )
    }
}

struct Nop {}

impl Nop {
    const OPCODE: u8 = 0xea;

    pub fn new() -> Self {
        Nop {}
    }
}

impl Operation for Nop {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += 1;
        cpu.cycles += 2;
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        format!("{:02X}        NOP      ", Self::OPCODE)
    }
}

/// Set carry flag.
struct Sec {}

impl Sec {
    const OPCODE: u8 = 0x38;

    pub fn new() -> Self {
        Sec {}
    }
}

impl Operation for Sec {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += 1;

        cpu.status.carry = true;

        cpu.cycles += 2;
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        format!("{:02X}        SEC      ", Self::OPCODE)
    }
}

/// Branch if carry flag set.
struct Bcs {
    /// Relative value to branch to.
    relative_value: u8,
}

impl Bcs {
    const OPCODE: u8 = 0xB0;

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

        if !cpu.status.carry {
            return;
        }

        cpu.program_counter += self.relative_value as u16;
        // TODO: Add cycles if it is a new page?
        cpu.cycles += 1;
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        format!("{:02X}        BCS      ", Self::OPCODE)
    }
}

/// Branch if carry flag clear.
struct Bcc {
    /// Relative value to branch to.
    relative_value: u8,
}

impl Bcc {
    const OPCODE: u8 = 0x90;

    pub fn new(cpu: &Cpu) -> Self {
        let relative_value = cpu.memory[(cpu.program_counter + 1) as usize];

        Bcc { relative_value }
    }
}

impl Operation for Bcc {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += 2;
        cpu.cycles += 2;

        if cpu.status.carry {
            return;
        }

        cpu.program_counter += self.relative_value as u16;
        // TODO: Add cycles if it is a new page?
        cpu.cycles += 1;
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        format!("{:02X}        BCC      ", Self::OPCODE)
    }
}

/// Branch if equal to zero.
struct Beq {
    /// Relative value to branch to.
    relative_value: u8,
}

impl Bcc {
    const OPCODE: u8 = 0xFO;

    pub fn new(cpu: &Cpu) -> Self {
        let relative_value = cpu.memory[(cpu.program_counter + 1) as usize];

        Bcc { relative_value }
    }
}

impl Operation for Bec {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += 2;
        cpu.cycles += 2;

        if !cpu.status.zero {
            return;
        }

        cpu.program_counter += self.relative_value as u16;
        // TODO: Add cycles if it is a new page?
        cpu.cycles += 1;
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        format!("{:02X}        BEC      ", Self::OPCODE)
    }
}

/// Clear carry flag.
struct Clc {}

impl Clc {
    const OPCODE: u8 = 0x18;

    pub fn new() -> Self {
        Clc {}
    }
}

impl Operation for Clc {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += 1;

        cpu.status.carry = false;

        cpu.cycles += 2;
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        format!("{:02X}        Clc      ", Self::OPCODE)
    }
}
