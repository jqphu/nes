/// This file contains the CPU logic.
/// Used http://nesdev.com/6502_cpu.txt as a reference.
///
/// The NMOS 65xx processors have 256 bytes of stack memory ranging from $0100 to $01FF.
use log::info;
use std::convert::From;

use crate::opcode::{self, *};

const MEMORY_SIZE_MAX: usize = 0xffff + 1;
pub type AddressSpace = [u8; MEMORY_SIZE_MAX];

/// State of the CPU.
///
/// For simplicity, we store the bank fixed to the CPU for now. As we build to a more advanced
/// structure we will move this outside of the CPU (e.g. bank switching).
///
/// TODO: Make these structs with certain operations available on them.
pub struct Cpu {
    /// Program counter.
    ///
    /// Low 8-bit is PCL, higher 8-bit is PCH.
    pub program_counter: u16,

    /// Stack.
    pub stack: Stack,

    /// Processor Status.
    ///
    /// Contains flags to denote the process status.
    pub status: ProcessorStatus,

    /// Accumulator.
    pub a: u8,

    /// Index register X.
    pub x: u8,

    /// Index register Y.
    pub y: u8,

    /// Memory.
    ///
    /// Limited to NROM thus only has 64 kibibytes.
    pub memory: AddressSpace,

    pub cycles: u64,
}

impl Cpu {
    const FIRST_16_KB_OF_ROM: usize = 0x8000;
    const LAST_16_KB_OF_ROM: usize = 0xC000;

    /// Create a new CPU from a NesFile.
    ///
    /// TODO: This is a little leaky, the CPU shouldn't know about the NES File Format but instead a
    /// third-party service should know about both the NES File Format and the CPU to initialize the
    /// state of the CPU and let it run.
    pub fn new(nes_file: crate::ines::NesFile) -> Self {
        // Power up state derived from http://wiki.nesdev.com/w/index.php/CPU_power_up_state.
        let mut cpu = Cpu {
            // Hard coded to start at ROM.
            program_counter: 0xc000,
            stack: Stack::new(),
            status: ProcessorStatus::new(),
            a: 0,
            x: 0,
            y: 0,
            memory: [0; MEMORY_SIZE_MAX],
            cycles: 7,
        };

        cpu.memory[Cpu::FIRST_16_KB_OF_ROM..Cpu::LAST_16_KB_OF_ROM]
            .copy_from_slice(&nes_file.prg_rom);
        cpu.memory[Cpu::LAST_16_KB_OF_ROM..].copy_from_slice(&nes_file.prg_rom);

        cpu
    }

    /// Start running!
    pub fn run(&mut self) {
        loop {
            let operation = opcode::next(self);
            info!(
                "{:X}  {}  \tA:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP: {:02X} CYC: {}",
                self.program_counter,
                &operation.dump(self),
                self.a,
                self.x,
                self.y,
                u8::from(&self.status),
                self.stack.as_stack_offset(),
                self.cycles
            );

            operation.execute(self);
        }
    }
}

/// A 8 bit register that has the processor state.
/// TODO: Expand this.
pub struct ProcessorStatus {
    /// Carry (C) Flag
    pub carry: bool,

    /// Zero (Z) Flag
    pub zero: bool,

    /// Interrupt Disable (I) Flag
    pub interrupt_disable: bool,

    /// Overflow Flag
    pub overflow: bool,

    /// Bit 4, not used by CPU.
    pub b_flag: bool,

    /// Negative flag.
    pub negative: bool,

    /// Decimal flag. Should not ever be used in nes.
    pub decimal: bool,
}

impl ProcessorStatus {
    const NEGATIVE_MASK: u8 = 0b1000_0000;
    const OVERFLOW_MASK: u8 = 0b0100_0000;
    fn new() -> Self {
        ProcessorStatus {
            carry: false,
            zero: false,
            b_flag: false,
            interrupt_disable: true,
            overflow: false,
            negative: false,
            decimal: false,
        }
    }

    pub fn update_load(&mut self, value: u8) {
        self.zero = value == 0;
        self.negative = value & ProcessorStatus::NEGATIVE_MASK == ProcessorStatus::NEGATIVE_MASK;
    }

    pub fn update_bit(&mut self, value: u8) {
        self.update_load(value);
        self.overflow = value & ProcessorStatus::OVERFLOW_MASK == ProcessorStatus::OVERFLOW_MASK;
    }
}

impl From<&ProcessorStatus> for u8 {
    fn from(src: &ProcessorStatus) -> u8 {
        // Upper bit of b flag (bit 5 of status) is always 1.
        let b_flag_upper = 1u8;

        (src.carry as u8)
            | (src.zero as u8) << 1
            | (src.interrupt_disable as u8) << 2
            | (src.decimal as u8) << 3
            | (src.b_flag as u8) << 4
            | b_flag_upper << 5
            | (src.overflow as u8) << 6
            | (src.negative as u8) << 7
    }
}

/// Stack starts at 0x1000.
pub struct Stack {
    // Address of the next free element in the stack (absolute address).
    stack_pointer: usize,
}

impl Stack {
    fn new() -> Self {
        Stack {
            stack_pointer: 0x1000 + 0xFD,
        }
    }

    /// Returns the expected value in cpu register which is an offset to $1000.
    fn as_stack_offset(&self) -> u8 {
        (self.stack_pointer - 0x1000) as u8
    }

    pub fn push_addr(&mut self, memory: &mut AddressSpace, addr: u16) {
        let (pcl, pch) = addr_to_bytes(addr);

        memory[self.stack_pointer] = pch;
        memory[self.stack_pointer - 1] = pcl;

        self.stack_pointer -= 2;
    }

    pub fn push(&mut self, memory: &mut AddressSpace, value: u8) {
        memory[self.stack_pointer] = value;
        self.stack_pointer -= 1;
    }

    pub fn pop(&mut self, memory: &mut AddressSpace) -> u8 {
        let value = memory[self.stack_pointer + 1];

        self.stack_pointer += 1;

        value
    }

    pub fn pop_addr(&mut self, memory: &mut AddressSpace) -> (u8, u8) {
        let pcl = memory[self.stack_pointer + 1];
        let pch = memory[self.stack_pointer + 2];

        self.stack_pointer += 2;

        (pcl, pch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ines;
    use anyhow::Result;
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    const LOG_FILENAME: &str = "test/nestest.log";
    const WORKING_UP_TO_LINE: u32 = 69;

    #[test]
    fn test_until_fail() -> Result<()> {
        let nes_file = ines::NesFile::new("test/nestest.nes".to_string())?;
        let mut cpu = Cpu::new(nes_file);
        let f = File::open(LOG_FILENAME)?;
        let f = BufReader::new(f);

        let mut counter = 1u32;

        for line in f.lines() {
            let line = line.unwrap();
            let operation = opcode::next(&cpu);

            let operation_output = format!("{:04X}  {}", cpu.program_counter, operation.dump(&cpu));
            if !line.contains(&operation_output) {
                println!("Expected output: {}", line);
                println!("Received output: {}", operation_output);
                panic!("Mismatch in operation state.");
            }

            let cpu_state_output = format!(
                "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
                cpu.a,
                cpu.x,
                cpu.y,
                u8::from(&cpu.status),
                cpu.stack.as_stack_offset(),
            );

            let cyc = format!("CYC:{}", cpu.cycles);

            if !line.contains(&cpu_state_output) || !line.contains(&cyc) {
                println!("Expected output: {}", line);
                println!("Received output: {} __ {}", cpu_state_output, cyc);
                panic!("Mismatch in cpu state.");
            }

            operation.execute(&mut cpu);

            counter += 1;

            // Don't test further than this line.
            if counter == WORKING_UP_TO_LINE {
                break;
            }
        }
        Ok(())
    }
}
