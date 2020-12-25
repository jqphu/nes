/// This file contains the CPU logic.
/// Used http://nesdev.com/6502_cpu.txt as a reference.
///
/// The NMOS 65xx processors have 256 bytes of stack memory ranging from $0100 to $01FF.
use log::info;
use std::convert::From;

use crate::opcode;

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

    /// Stack pointer.
    ///
    /// Offset from the stack page. I.e. 0x1000 + stack_pointer is currently the top of the stack.
    stack_pointer: u8,

    /// Processor Status.
    ///
    /// Contains flags to denote the process status.
    pub status: ProcessorStatus,

    /// Accumulator.
    accumulator: u8,

    /// Index register X.
    pub x: u8,

    /// Index register Y.
    y: u8,

    /// Memory.
    ///
    /// Limited to NROM thus only has 64 kibibytes.
    pub memory: [u8; Cpu::MEMORY_SIZE_MAX],

    pub cycles: u64,
}

impl Cpu {
    const MEMORY_SIZE_MAX: usize = 0xffff + 1;

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
            stack_pointer: 0xfd,
            status: ProcessorStatus::new(),
            accumulator: 0,
            x: 0,
            y: 0,
            memory: [0; Cpu::MEMORY_SIZE_MAX],
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
                "{:X}  {} \t A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP: {:02X} CYC: {}",
                self.program_counter,
                &operation.dump(self),
                self.accumulator,
                self.x,
                self.y,
                u8::from(&self.status),
                self.stack_pointer,
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
    carry: bool,

    /// Zero (Z) Flag
    zero: bool,

    /// Interrupt Disable (I) Flag
    interrupt_disable: bool,

    /// Overflow Flag
    overflow: bool,

    /// Bit 4, not used by CPU.
    b_flag: bool,

    /// Negative flag.
    negative: bool,
    // Decimal flag not used on NES.
}

impl ProcessorStatus {
    fn new() -> Self {
        ProcessorStatus {
            carry: false,
            zero: false,
            b_flag: false,
            interrupt_disable: true,
            overflow: false,
            negative: false,
        }
    }

    pub fn update(&mut self, value: u8) {
        if value == 0 {
            self.zero = true;
        }

        // Bit 7 is set
        if (value & 0b10000000) == 1 {
            self.negative = true;
        }
    }
}

impl From<&ProcessorStatus> for u8 {
    fn from(src: &ProcessorStatus) -> u8 {
        // Decimal is not used in NES.
        let decimal = 0u8;

        // Upper bit of b flag (bit 5 of status) is always 1.
        let b_flag_upper = 1u8;

        (src.carry as u8)
            | (src.zero as u8) << 1
            | (src.interrupt_disable as u8) << 2
            | decimal << 3
            | (src.b_flag as u8) << 4
            | b_flag_upper << 5
            | (src.overflow as u8) << 6
            | (src.negative as u8) << 7
    }
}
