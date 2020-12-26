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

    if let Some(nop) = Nop::new(opcode) {
        return Box::new(nop);
    }

    if let Some(bit) = Bit::new(opcode, cpu) {
        return Box::new(bit);
    }

    panic!("Unexpected opcode {:02X}", opcode);
}
