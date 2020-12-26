use crate::cpu::Cpu;
use crate::opcode::addressing_mode::{AddRegister, AddressMode};
use crate::opcode::*;
use std::string::ToString;

pub struct Load {
    /// Addressing mode.
    mode: AddressMode,

    /// Which register to load from.
    register: Register,

    /// Convenience to store the op code.
    opcode: u8,
}

impl Load {
    pub fn new(opcode: u8, cpu: &Cpu) -> Option<Self> {
        let register = Load::get_register(opcode)?;
        Some(Load {
            mode: Load::get_mode(opcode, cpu),
            register,
            opcode: opcode,
        })
    }

    /// Get the register from the Load opcode.
    fn get_register(opcode: u8) -> Option<Register> {
        match opcode {
            0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => Some(Register::A),
            0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => Some(Register::X),
            0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => Some(Register::Y),
            _ => None,
        }
    }

    /// Get the mode from the opcode.
    fn get_mode(opcode: u8, cpu: &Cpu) -> AddressMode {
        let pc = cpu.program_counter as usize;
        let value = cpu.memory[pc + 1];

        match opcode {
            0xA9 | 0xA2 | 0xA0 => AddressMode::Immediate { value },
            0xA5 | 0xA6 | 0xA4 => AddressMode::ZeroPage {
                register: AddRegister::None,
                offset: value,
            },
            0xB5 | 0xB4 => AddressMode::ZeroPage {
                register: AddRegister::X,
                offset: value,
            },
            0xB6 => AddressMode::ZeroPage {
                register: AddRegister::Y,
                offset: value,
            },
            0xAD | 0xAE | 0xAC => AddressMode::Absolute {
                register: AddRegister::None,
                address: bytes_to_addr(value, cpu.memory[pc + 2]),
            },
            0xBD | 0xBC => AddressMode::Absolute {
                register: AddRegister::X,
                address: bytes_to_addr(value, cpu.memory[pc + 2]),
            },
            0xB9 | 0xBE => AddressMode::Absolute {
                register: AddRegister::X,
                address: bytes_to_addr(value, cpu.memory[pc + 2]),
            },
            0xA1 => AddressMode::Indirect {
                register: AddRegister::X,
                address_to_read_indirect: bytes_to_addr(value, cpu.memory[pc + 2]),
            },
            0xB1 => AddressMode::Indirect {
                register: AddRegister::Y,
                address_to_read_indirect: bytes_to_addr(value, cpu.memory[pc + 2]),
            },
            _ => panic!("Unexpected opcode {:X}", opcode),
        }
    }

    fn get_cycles(&self, cpu: &Cpu) -> u64 {
        match &self.mode {
            AddressMode::Immediate { value: _ } => 2,
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
                address,
            } => 4 + is_on_different_pages(*address, *address + cpu.x as u16) as u64,
            AddressMode::Indirect {
                register: AddRegister::X,
                address_to_read_indirect: _,
            } => 6,
            AddressMode::Indirect {
                register: AddRegister::Y,
                address_to_read_indirect: address,
            } => 5 + is_on_different_pages(*address, *address + cpu.y as u16) as u64,
            _ => panic!("Unexpected!"),
        }
    }

    fn get_bytes(&self) -> u64 {
        match &self.mode {
            AddressMode::Immediate { value: _ } => 2,
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

impl Operation for Load {
    /// JMP simply moves to the address.
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += self.get_bytes() as u16;
        cpu.cycles += self.get_cycles(cpu);
        let value = self.mode.to_value(cpu);

        let target_cpu = match self.register {
            Register::X => &mut cpu.x,
            Register::Y => &mut cpu.y,
            Register::A => &mut cpu.a,
        };

        *target_cpu = value;
        cpu.status.update_load(*target_cpu);
    }

    fn dump(&self, cpu: &Cpu) -> String {
        format!(
            "{:02X} {}     LD{} {}",
            self.opcode,
            self.mode.value_to_string(),
            self.register.to_string(),
            self.mode.to_string(cpu)
        )
    }
}
