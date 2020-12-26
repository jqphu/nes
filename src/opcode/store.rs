use crate::cpu::Cpu;
use crate::opcode::addressing_mode::{AddRegister, AddressMode};
use crate::opcode::*;
use std::string::ToString;

pub struct Store {
    /// Addressing mode.
    mode: AddressMode,

    /// Which register to load from.
    register: Register,

    /// Convenience to store the op code.
    opcode: u8,
}

impl Store {
    pub fn new(opcode: u8, cpu: &Cpu) -> Option<Self> {
        let register = Store::get_register(opcode)?;
        Some(Store {
            mode: Store::get_mode(opcode, cpu),
            register,
            opcode: opcode,
        })
    }

    /// Get the register from the Store opcode.
    fn get_register(opcode: u8) -> Option<Register> {
        match opcode {
            0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => Some(Register::A),
            0x86 | 0x96 | 0x8E => Some(Register::X),
            0x84 | 0x94 | 0x8C => Some(Register::Y),
            _ => None,
        }
    }

    /// Get the mode from the opcode.
    fn get_mode(opcode: u8, cpu: &Cpu) -> AddressMode {
        let pc = cpu.program_counter as usize;
        let value = cpu.memory[pc + 1];

        match opcode {
            0x85 | 0x86 | 0x84 => AddressMode::ZeroPage {
                register: AddRegister::None,
                offset: value,
            },
            0x95 | 0x94 => AddressMode::ZeroPage {
                register: AddRegister::X,
                offset: value,
            },
            0x96 => AddressMode::ZeroPage {
                register: AddRegister::Y,
                offset: value,
            },
            0x8D | 0x8E | 0x8C => AddressMode::Absolute {
                register: AddRegister::None,
                address: bytes_to_addr(value, cpu.memory[pc + 2]),
            },
            0x9D => AddressMode::Absolute {
                register: AddRegister::X,
                address: bytes_to_addr(value, cpu.memory[pc + 2]),
            },
            0x99 => AddressMode::Absolute {
                register: AddRegister::X,
                address: bytes_to_addr(value, cpu.memory[pc + 2]),
            },
            0x81 => AddressMode::Indirect {
                register: AddRegister::X,
                address_to_read_indirect: bytes_to_addr(value, cpu.memory[pc + 2]),
            },
            0x91 => AddressMode::Indirect {
                register: AddRegister::Y,
                address_to_read_indirect: bytes_to_addr(value, cpu.memory[pc + 2]),
            },
            _ => panic!("Unexpected opcode {:X}", opcode),
        }
    }

    fn get_cycles(&self, _cpu: &Cpu) -> u64 {
        match &self.mode {
            AddressMode::ZeroPage {
                register: AddRegister::None,
                offset: _,
            } => 3,
            AddressMode::ZeroPage {
                register: _,
                offset: _,
            } => 4,
            AddressMode::Absolute {
                register: AddRegister::None,
                address: _,
            } => 4,
            AddressMode::Absolute {
                register: _,
                address: _,
            } => 5,
            AddressMode::Indirect {
                register: _,
                address_to_read_indirect: _,
            } => 6,
            _ => panic!("Unexpected!"),
        }
    }

    fn get_bytes(&self) -> u64 {
        match &self.mode {
            AddressMode::ZeroPage {
                register: _,
                offset: _,
            } => 2,
            AddressMode::Absolute {
                register: _,
                address: _,
            } => 3,
            AddressMode::Indirect {
                register: _,
                address_to_read_indirect: _,
            } => 2,
            _ => panic!("Unexpected!"),
        }
    }
}

impl Operation for Store {
    /// JMP simply moves to the address.
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += self.get_bytes() as u16;
        cpu.cycles += self.get_cycles(cpu);
        let addr = self.mode.to_addr(cpu).unwrap();

        let value = match self.register {
            Register::X => cpu.x,
            Register::Y => cpu.y,
            Register::A => cpu.a,
        };

        cpu.memory[addr as usize] = value;
    }

    fn dump(&self, cpu: &Cpu) -> String {
        format!(
            "{:02X} {}     ST{} {}",
            self.opcode,
            self.mode.value_to_string(),
            self.register.to_string(),
            self.mode.to_string(cpu)
        )
    }
}
