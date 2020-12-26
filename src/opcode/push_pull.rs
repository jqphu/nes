use crate::cpu::Cpu;
use crate::opcode::Operation;
use std::string::ToString;

enum Data {
    Accumulator,
    ProcessorStatus,
}

impl ToString for Data {
    fn to_string(&self) -> String {
        match self {
            Data::Accumulator => "A",
            Data::ProcessorStatus => "P",
        }
        .to_string()
    }
}

pub struct Push {
    opcode: u8,

    /// What data to push.
    data: Data,
}

impl Push {
    const BYTES: u16 = 1;
    const CYCLES: u64 = 3;

    pub fn new(opcode: u8) -> Option<Self> {
        match opcode {
            0x48 => Some(Push {
                opcode,
                data: Data::Accumulator,
            }),
            0x08 => Some(Push {
                opcode,
                data: Data::ProcessorStatus,
            }),
            _ => None,
        }
    }
}

impl Operation for Push {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += Self::BYTES;
        cpu.cycles += Self::CYCLES;

        let value = match self.data {
            Data::Accumulator => cpu.a,
            Data::ProcessorStatus => u8::from(cpu.status.clone()),
        };

        cpu.stack.push(&mut cpu.memory, value);
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        format!(
            "{:02X}        PH{}     ",
            self.opcode,
            self.data.to_string()
        )
    }
}

pub struct Pull {
    opcode: u8,

    /// What data to push.
    data: Data,
}

impl Pull {
    const BYTES: u16 = 1;
    const CYCLES: u64 = 4;

    pub fn new(opcode: u8) -> Option<Self> {
        match opcode {
            0x68 => Some(Pull {
                opcode,
                data: Data::Accumulator,
            }),
            0x28 => Some(Pull {
                opcode,
                data: Data::ProcessorStatus,
            }),
            _ => None,
        }
    }
}

impl Operation for Pull {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.program_counter += Self::BYTES;
        cpu.cycles += Self::CYCLES;

        let value = cpu.stack.pop(&mut cpu.memory);

        match self.data {
            Data::Accumulator => {
                cpu.a = value;
                cpu.status.update_load(cpu.a);
            }
            Data::ProcessorStatus => cpu.status = value.into(),
        };
    }

    fn dump(&self, _cpu: &Cpu) -> String {
        format!(
            "{:02X}        PL{}     ",
            self.opcode,
            self.data.to_string()
        )
    }
}
