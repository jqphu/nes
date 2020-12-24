use anyhow::Result;
use clap::Clap;
use env_logger;
use log::info;

mod ines;

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

    let _result = ines::NesFile::new(opts.rom)?;

    Ok(())
}
