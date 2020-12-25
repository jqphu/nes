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
        0x4C | 0x6C => Box::new(Jmp::new(opcode_raw, cpu)),
        0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => Box::new(Ldx::new(opcode_raw, cpu)),
        0x86 | 0x96 | 0x8E => Box::new(Stx::new(opcode_raw, cpu)),
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
            self.addr_to_bytes().0,
            self.addr_to_bytes().1,
            addr_string
        )
    }
}

impl Jmp {
    pub fn new(opcode_raw: u8, cpu: &Cpu) -> Self {
        let pc = cpu.program_counter as usize;
        let addr = Jmp::bytes_to_addr(cpu.memory[pc + 1], cpu.memory[pc + 2]);
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
    /// Assuming little endian, merge the two u8 values into a u16 value.
    /// For some reason, the orders are reversed?
    /// TODO: Figure out Why?
    fn bytes_to_addr(first: u8, second: u8) -> u16 {
        ((second as u16) << 8) | first as u16
    }

    fn addr_to_bytes(&self) -> (u8, u8) {
        (self.addr as u8, (self.addr >> 8) as u8)
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
