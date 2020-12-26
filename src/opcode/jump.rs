use crate::cpu::Cpu;
use crate::opcode::addressing_mode::{AddRegister, AddressMode};
use crate::opcode::*;

pub struct Jmp {
    opcode: u8,

    // Absolute or Indirect.
    mode: AddressMode,
}

impl Jmp {
    pub fn new(opcode: u8, cpu: &Cpu) -> Option<Self> {
        let pc = cpu.program_counter as usize;
        let address = bytes_to_addr(cpu.memory[pc + 1], cpu.memory[pc + 2]);
        match opcode {
            // Absolute
            0x4C => Some(Jmp {
                opcode,
                mode: AddressMode::Absolute {
                    register: AddRegister::None,
                    address,
                },
            }),

            0x6C => Some(Jmp {
                opcode,
                mode: AddressMode::Indirect {
                    register: AddRegister::None,
                    address_to_read_indirect: address,
                },
            }),
            _ => None,
        }
    }

    fn get_cycles(&self, _cpu: &Cpu) -> u64 {
        match &self.mode {
            AddressMode::Absolute {
                register: _,
                address: _,
            } => 3,
            AddressMode::Indirect {
                register: _,
                address_to_read_indirect: _,
            } => 5,
            _ => panic!("unexpected!"),
        }
    }
}

impl Operation for Jmp {
    /// JMP simply moves to the address.
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter = self.mode.to_addr(cpu).unwrap();
        cpu.cycles += self.get_cycles(cpu);
    }

    fn dump(&self, cpu: &Cpu) -> String {
        format!(
            "{:02X} {}  JMP {}",
            self.opcode,
            self.mode.value_to_string(),
            self.mode.to_string(cpu),
        )
    }
}

pub struct Jsr {
    // Absolute.
    mode: AddressMode,
}

impl Jsr {
    const OPCODE: u8 = 0x20;
    const BYTES: u16 = 3;
    const CYCLES: u64 = 6;

    pub fn new(opcode: u8, cpu: &Cpu) -> Option<Self> {
        if opcode != Jsr::OPCODE {
            return None;
        }

        let pc = cpu.program_counter as usize;
        let address = bytes_to_addr(cpu.memory[pc + 1], cpu.memory[pc + 2]);
        Some(Jsr {
            mode: AddressMode::Absolute {
                register: AddRegister::None,
                address,
            },
        })
    }
}

impl Operation for Jsr {
    fn execute(&self, cpu: &mut Cpu) {
        // Jsr always 3 bytes. Push return address - 1.
        let return_address = cpu.program_counter + Jsr::BYTES - 1;

        // Push onto the stack the return address.
        cpu.stack.push_addr(&mut cpu.memory, return_address);

        cpu.program_counter = self.mode.to_addr(cpu).unwrap();

        // Always 6 cycles for a JSR
        cpu.cycles += Jsr::CYCLES;
    }

    fn dump(&self, cpu: &Cpu) -> String {
        format!(
            "{:02X} {}  JSR {}",
            Self::OPCODE,
            self.mode.value_to_string(),
            self.mode.to_string(cpu),
        )
    }
}

pub struct Rts {}

impl Rts {
    const OPCODE: u8 = 0x60;
    const BYTES: u16 = 1;
    const CYCLES: u64 = 6;

    pub fn new(opcode: u8) -> Option<Self> {
        if opcode != Rts::OPCODE {
            return None;
        }

        Some(Rts {})
    }
}

impl Operation for Rts {
    fn execute(&self, cpu: &mut Cpu) {
        let (pcl, pch) = cpu.stack.pop_addr(&mut cpu.memory);

        let return_address = bytes_to_addr(pcl, pch);

        cpu.program_counter = return_address;

        cpu.program_counter += Rts::BYTES;
        cpu.cycles += Rts::CYCLES;
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        format!("{:02X}        RTS     ", Self::OPCODE)
    }
}
