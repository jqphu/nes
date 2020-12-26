use anyhow::{anyhow, Result};
use log::debug;
use std::fs::File;
use std::io::Read;

/// iNes Structure.
pub struct NesFile {
    // Header for the given file.
    // header: Header,

    // PrgRom buffer.
    pub prg_rom: Vec<u8>,
}

/// Header of a iNes ROM
struct Header {
    /// PRG Rom Size in multples of 16kB.
    ///
    /// This is used by the CPU.
    prg_rom_multiple_size: u8,
    // TODO: More flags :)
}

impl Header {
    /// Number of bytes in the header.
    const HEADER_SIZE_BYTES: usize = 16;

    /// 16 KiB is the multiple.
    const PRG_ROM_MULTIPLE: usize = 16384;

    /// Construct a header struct from the raw 16 header bytes.
    fn new(header: [u8; Self::HEADER_SIZE_BYTES]) -> Result<Self> {
        if header[0] != b'N' || header[1] != b'E' || header[2] != b'S' || header[3] != 0x1A {
            return Err(anyhow!("Invalid file magic {:?}.", header));
        }

        let result = Header {
            prg_rom_multiple_size: header[4],
        };

        for &byte in &header[6..] {
            if byte != 0 {
                return Err(anyhow!("Unsupported nes file format."));
            }
        }

        Ok(result)
    }

    /// Return the prg rom size in bytes.
    fn get_prg_rom_size(&self) -> usize {
        self.prg_rom_multiple_size as usize * Self::PRG_ROM_MULTIPLE
    }
}

impl NesFile {
    pub fn new(filename: String) -> Result<Self> {
        debug!("Parsing filename {}", filename);

        let mut f = File::open(&filename)?;

        let header = {
            // Initialized immediately after.
            let mut header_raw: [u8; Header::HEADER_SIZE_BYTES] =
                unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            f.read(&mut header_raw)?;

            debug!("Received header: {:x?}", &header_raw);

            Header::new(header_raw)?
        };

        let prg_rom = {
            let mut buffer = vec![0; header.get_prg_rom_size()];

            debug!("Rom size is: {}", &header.get_prg_rom_size());

            f.take(header.get_prg_rom_size() as u64).read(&mut buffer)?;

            debug!("Received prg rom: {:x?}", &buffer);

            buffer
        };

        Ok(NesFile { prg_rom })
    }
}
