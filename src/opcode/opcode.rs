/// This file contains the OpCodes and their implementations.
use crate::cpu::Cpu;
use crate::opcode::*;
use log::debug;

pub trait Operation {
    /// Execute the opcode.
    fn execute(&self, cpu: &mut Cpu);

    /// Dump the opcode with the read values.
    fn dump(&self, cpu: &Cpu) -> String;
}

/// Read in the next opcode and set up PC.
pub fn next(cpu: &Cpu) -> Box<dyn Operation> {
    let pc = cpu.program_counter as usize;
    let opcode = cpu.memory[pc];

    if let Some(branch) = Branch::new(opcode, cpu) {
        return Box::new(branch);
    }

    if let Some(flag) = Flag::new(opcode) {
        return Box::new(flag);
    }

    match opcode {
        Nop::OPCODE => Box::new(Nop::new()),
        0x4C | 0x6C => Box::new(Jmp::new(opcode, cpu)),
        Jsr::OPCODE => Box::new(Jsr::new(cpu)),
        0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => Box::new(Ldx::new(opcode, cpu)),
        0x86 | 0x96 | 0x8E => Box::new(Stx::new(opcode, cpu)),
        0x85 => Box::new(Sta::new(opcode, cpu)),
        0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => Box::new(Lda::new(opcode, cpu)),
        Bit::OPCODE => Box::new(Bit::new(cpu)),
        Rts::OPCODE => Box::new(Rts::new()),
        _ => panic!("Unsupported {:X}", opcode),
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
            "{:02X} {:02X} {:02X}  JMP {}",
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

        cpu.status.update_load(cpu.x);
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        let addr_string = match self.mode {
            AddressingMode::Immediate => format!("#${:02X}", self.value),
            AddressingMode::Absolute => format!("${:02X}", self.value),
            _ => panic!("Unsupproted!"),
        };

        format!(
            "{:02X} {:02X}     LDX {}",
            self.opcode_raw, self.value, addr_string
        )
    }
}

struct Lda {
    opcode_raw: u8,

    /// Only Absolute and Indirect are valid modes.
    mode: AddressingMode,

    /// Number of cycles this operation takes.
    cycles: u8,

    /// Value to load.
    value: u8,
}

/// TODO: Generalize addressing mode to a separate module.
impl Lda {
    pub fn new(opcode_raw: u8, cpu: &Cpu) -> Self {
        let pc = cpu.program_counter as usize;
        let value = cpu.memory[pc + 1];
        match opcode_raw {
            // Immediate.
            0xA9 => Lda {
                opcode_raw,
                mode: AddressingMode::Immediate,
                cycles: 2,
                value,
            },
            _ => panic!("Unsupported {}", opcode_raw),
        }
    }
}

impl Operation for Lda {
    /// JMP simply moves to the address.
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += self.mode.get_bytes() as u16;
        cpu.a = self.value;

        cpu.cycles += u64::from(self.cycles);
        cpu.status.update_load(cpu.a);
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        let addr_string = match self.mode {
            AddressingMode::Immediate => format!("#${:02X}", self.value),
            _ => panic!("Unsupproted!"),
        };

        format!(
            "{:02X} {:02X}     LDA {}",
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
            "{:02X} {:02X}     STX {}",
            self.opcode_raw, self.zero_page_addr, addr_string
        )
    }
}

struct Sta {
    opcode_raw: u8,

    /// Only Absolute and Indirect are valid modes.
    mode: AddressingMode,

    /// Number of cycles this operation takes.
    cycles: u8,

    /// Address to store value in register x.
    zero_page_addr: u8,
}

impl Sta {
    pub fn new(opcode_raw: u8, cpu: &Cpu) -> Self {
        let pc = cpu.program_counter as usize;
        let zero_page_addr = cpu.memory[pc + 1];
        match opcode_raw {
            // Immediate.
            0x85 => Sta {
                opcode_raw,
                mode: AddressingMode::ZeroPage,
                cycles: 3,
                zero_page_addr,
            },
            _ => panic!("Unsupported {}", opcode_raw),
        }
    }
}

impl Operation for Sta {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += self.mode.get_bytes() as u16;
        cpu.memory[self.zero_page_addr as usize] = cpu.a;
        cpu.cycles += u64::from(self.cycles);
    }

    fn dump(&self, cpu: &Cpu) -> String {
        let addr_string = match self.mode {
            AddressingMode::ZeroPage => format!(
                "${:02X} = {:02X}",
                self.zero_page_addr, cpu.memory[self.zero_page_addr as usize]
            ),
            _ => panic!("Unsupported!"),
        };

        format!(
            "{:02X} {:02X}     STA {}",
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
        // Jsr always 3 bytes. Push return address - 1.
        let return_address = cpu.program_counter + 3 - 1;

        let (pcl, pch) = addr_to_bytes(return_address);

        // Push onto the stack the return address.
        cpu.stack.push(&mut cpu.memory, &[pch, pcl]);

        cpu.program_counter = self.addr;

        // Always 6 cycles for a JSR
        cpu.cycles += 6;
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        let addr_string = format!("${:X}", self.addr);

        format!(
            "{:02X} {:02X} {:02X}  JSR {}",
            Self::OPCODE,
            addr_to_bytes(self.addr).0,
            addr_to_bytes(self.addr).1,
            addr_string
        )
    }
}

struct Rts {}

impl Rts {
    const OPCODE: u8 = 0x60;

    pub fn new() -> Self {
        Rts {}
    }
}

impl Operation for Rts {
    fn execute(&self, cpu: &mut Cpu) {
        let (pch, pcl) = cpu.stack.pop_addr(&mut cpu.memory);

        // TODO: Fix ordering of the arguments.
        let return_address = bytes_to_addr(pcl, pch);

        cpu.program_counter = return_address;

        // Add 1 for the RTS command.
        cpu.program_counter += 1;

        debug!("Returning to {:02X}", cpu.program_counter);
        cpu.cycles += 6
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        format!("{:02X}        RTS     ", Self::OPCODE)
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
        format!("{:02X}        NOP     ", Self::OPCODE)
    }
}

/// Bit test.
///
/// And the memory with what is in the accumulator and set flags.
struct Bit {
    /// Address of the memory to test.
    zero_page_addr: u8,
}

impl Bit {
    const OPCODE: u8 = 0x24;

    pub fn new(cpu: &Cpu) -> Self {
        let zero_page_addr = cpu.memory[(cpu.program_counter + 1) as usize];

        Bit { zero_page_addr }
    }
}

impl Operation for Bit {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += 2;
        cpu.cycles += 3;

        let test_value = cpu.memory[self.zero_page_addr as usize];

        let result = test_value & cpu.a;
        cpu.status.update_bit(result);
    }

    fn dump(&self, cpu: &Cpu) -> String {
        format!(
            "{:02X} {:02X}     BIT ${:02X} = {:02X}   ",
            Self::OPCODE,
            self.zero_page_addr,
            self.zero_page_addr,
            cpu.memory[self.zero_page_addr as usize]
        )
    }
}
