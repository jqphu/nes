/// This file contains the OpCodes and their implementations.
use crate::cpu::Cpu;
use crate::opcode::*;
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

    match opcode {
        Nop::OPCODE => Box::new(Nop::new()),
        Bit::OPCODE => Box::new(Bit::new(cpu)),
        _ => panic!("Unsupported {:X}", opcode),
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
