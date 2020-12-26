use anyhow::Result;
use clap::Clap;
use log::info;

mod cpu;
mod ines;
mod opcode;

/// Basic emulator for the NES.
#[derive(Clap)]
#[clap(version = "0.0.1", author = "Justin Phu. <justinqphu@gmail.com>")]
struct Opts {
    /// Nes rom to test.
    rom: String,
}

fn main() -> Result<()> {
    env_logger::init();

    let opts: Opts = Opts::parse();

    info!("Loading ROM \"{}\"", &opts.rom);

    let nes_file = ines::NesFile::new(opts.rom)?;

    let mut cpu = cpu::Cpu::new(nes_file);

    cpu.run();

    Ok(())
}
