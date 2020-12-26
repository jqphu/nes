mod addressing_mode;
mod branch;
mod flag;
mod jump;
mod load;
mod store;

use crate::cpu::Cpu;
use crate::opcode::addressing_mode::{AddRegister, AddressMode};

pub use branch::*;
pub use flag::*;
pub use jump::*;
pub use load::*;
pub use store::*;

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

    if let Some(load) = Load::new(opcode, cpu) {
        return Box::new(load);
    }

    if let Some(store) = Store::new(opcode, cpu) {
        return Box::new(store);
    }

    if let Some(jmp) = Jmp::new(opcode, cpu) {
        return Box::new(jmp);
    }

    if let Some(jsr) = Jsr::new(opcode, cpu) {
        return Box::new(jsr);
    }

    if let Some(rts) = Rts::new(opcode) {
        return Box::new(rts);
    }

    if let Some(nop) = Nop::new(opcode) {
        return Box::new(nop);
    }

    if let Some(bit) = Bit::new(opcode, cpu) {
        return Box::new(bit);
    }

    panic!("Unexpected opcode {:02X}", opcode);
}

/// Each page is 256 bytes.
const PAGE_SIZE: u16 = 0x100;

/// Assuming little endian, merge the two u8 values into a u16 value.
/// For some reason, the orders are reversed?
/// TODO: Figure out Why?
pub fn bytes_to_addr(first: u8, second: u8) -> u16 {
    ((second as u16) << 8) | first as u16
}

pub fn addr_to_bytes(addr: u16) -> (u8, u8) {
    (addr as u8, (addr >> 8) as u8)
}

/// If the src address is on a different page to dest address.
fn is_on_different_pages(src: u16, dest: u16) -> bool {
    src / PAGE_SIZE != dest / PAGE_SIZE
}

enum Register {
    A,
    X,
    Y,
}

impl ToString for Register {
    fn to_string(&self) -> String {
        match self {
            Register::A => "A",
            Register::X => "X",
            Register::Y => "Y",
        }
        .to_string()
    }
}

struct Nop {}

impl Nop {
    const OPCODE: u8 = 0xEA;
    const BYTES: u16 = 1;
    const CYCLES: u64 = 2;

    pub fn new(opcode: u8) -> Option<Self> {
        if opcode != Nop::OPCODE {
            None
        } else {
            Some(Nop {})
        }
    }
}

impl Operation for Nop {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += Self::BYTES;
        cpu.cycles += Self::CYCLES;
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        format!("{:02X}        NOP     ", Self::OPCODE)
    }
}

/// Bit test.
///
/// And the memory with what is in the accumulator and set flags.
struct Bit {
    opcode: u8,

    /// Address of the memory to test.
    mode: AddressMode,
}

impl Bit {
    pub fn new(opcode: u8, cpu: &Cpu) -> Option<Self> {
        let pc = cpu.program_counter as usize;
        let value = cpu.memory[pc + 1];

        match opcode {
            0x24 => Some(Bit {
                opcode,
                mode: AddressMode::ZeroPage {
                    register: AddRegister::None,
                    offset: value,
                },
            }),
            0x2C => {
                let address = bytes_to_addr(value, cpu.memory[pc + 2]);
                Some(Bit {
                    opcode,
                    mode: AddressMode::Absolute {
                        register: AddRegister::None,
                        address,
                    },
                })
            }
            _ => None,
        }
    }

    pub fn get_bytes(&self) -> u16 {
        match &self.mode {
            AddressMode::ZeroPage {
                register: _,
                offset: _,
            } => 2,
            AddressMode::Absolute {
                register: _,
                address: _,
            } => 3,
            _ => panic!("Unexpected!"),
        }
    }

    pub fn get_cycles(&self) -> u64 {
        match &self.mode {
            AddressMode::ZeroPage {
                register: _,
                offset: _,
            } => 3,
            AddressMode::Absolute {
                register: _,
                address: _,
            } => 4,
            _ => panic!("Unexpected!"),
        }
    }
}

impl Operation for Bit {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += self.get_bytes();
        cpu.cycles += self.get_cycles();

        let test_value = self.mode.to_value(cpu);
        let result = test_value & cpu.a;

        cpu.status.update_bit(result);
    }

    fn dump(&self, cpu: &Cpu) -> String {
        format!(
            "{:02X} {}     BIT {}   ",
            self.opcode,
            self.mode.value_to_string(),
            self.mode.to_string(cpu),
        )
    }
}
