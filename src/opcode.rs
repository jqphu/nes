/// This file contains the OpCodes and their implementations.
use crate::cpu::Cpu;

pub trait Operation {
    /// Execute the opcode.
    fn execute(&self, cpu: &mut Cpu);

    /// Dump the opcode with the read values.
    fn dump(&self) -> String;
}

/// Read in the next opcode and set up PC.
pub fn next(cpu: &Cpu) -> Box<dyn Operation> {
    let opcode = {
        let pc = cpu.program_counter as usize;
        let opcode_raw = cpu.memory[pc];

        match opcode_raw {
            0x4C | 0x6C => Jmp::new(opcode_raw, cpu),
            _ => panic!("Unsupported {}", opcode_raw),
        }
    };

    Box::new(opcode)
}

enum AddressingMode {
    Absolute,
    Indirect,
}

impl AddressingMode {
    /// Number of bytes an operation takes including opcode.
    /// http://obelisk.me.uk/6502/reference.html
    fn get_bytes(&self) -> usize {
        match &self {
            AddressingMode::Absolute => 3,
            AddressingMode::Indirect => 3,
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

    fn dump(&self) -> String {
        let addr_string = match self.mode {
            AddressingMode::Absolute => format!("${:X}", self.addr),
            AddressingMode::Indirect => panic!("Unsupproted!"),
        };

        format!(
            "{:X} {:X} {:X}  JMP  {}",
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
            0x4C => Jmp {
                opcode_raw,
                mode: AddressingMode::Absolute,
                cycles: 3,
                addr,
            },
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
