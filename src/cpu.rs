/// This file contains the CPU logic.
/// Used http://nesdev.com/6502_cpu.txt as a reference.
///
/// The NMOS 65xx processors have 256 bytes of stack memory ranging from $0100 to $01FF.
use log::info;

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
    status: ProcessorStatus,

    /// Accumulator.
    accumulator: u8,

    /// Index register X.
    x: u8,

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
            status: ProcessorStatus(0),
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
            info!("{:X}  {}", self.program_counter, &operation.dump());

            operation.execute(self);
        }
    }
}

/// A 8 bit register that has the processor state.
/// TODO: Expand this.
struct ProcessorStatus(u8);
