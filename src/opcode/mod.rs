mod addressing_mode;
mod branch;
mod flag;
mod opcode;

pub use branch::*;
pub use flag::*;
pub use opcode::*;

/// Assuming little endian, merge the two u8 values into a u16 value.
/// For some reason, the orders are reversed?
/// TODO: Figure out Why?
pub fn bytes_to_addr(first: u8, second: u8) -> u16 {
    ((second as u16) << 8) | first as u16
}

pub fn addr_to_bytes(addr: u16) -> (u8, u8) {
    (addr as u8, (addr >> 8) as u8)
}
