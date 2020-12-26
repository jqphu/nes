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

    if let Some(load) = Load::new(opcode, cpu) {
        return Box::new(load);
    }

    if let Some(store) = Store::new(opcode, cpu) {
        return Box::new(store);
    }

    match opcode {
        Nop::OPCODE => Box::new(Nop::new()),
        0x4C | 0x6C => Box::new(Jmp::new(opcode, cpu)),
        Jsr::OPCODE => Box::new(Jsr::new(cpu)),
        Bit::OPCODE => Box::new(Bit::new(cpu)),
        Rts::OPCODE => Box::new(Rts::new()),
        _ => panic!("Unsupported {:X}", opcode),
    }
}

/// Jmp OpCode.
struct Jmp {
    opcode_raw: u8,

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
        format!(
            "{:02X} {:02X} {:02X}  JMP {}",
            self.opcode_raw,
            addr_to_bytes(self.addr).0,
            addr_to_bytes(self.addr).1,
            format!("${:X}", self.addr),
        )
    }
}

impl Jmp {
    pub fn new(opcode_raw: u8, cpu: &Cpu) -> Self {
        let pc = cpu.program_counter as usize;
        let addr = bytes_to_addr(cpu.memory[pc + 1], cpu.memory[pc + 2]);
        match opcode_raw {
            // Absolute
            0x4C => Jmp {
                opcode_raw,
                cycles: 3,
                addr,
            },
            _ => panic!("Unexpected opcode"),
        }
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
