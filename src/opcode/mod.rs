mod addressing_mode;
mod branch;
mod flag;
mod load;
mod store;
mod opcode;
mod jump;

pub use branch::*;
pub use flag::*;
pub use load::*;
pub use store::*;
pub use opcode::*;
pub use jump::*;

/// Each page is 256 bytes.
const PAGE_SIZE: u16 = 0x100;

/// Assuming little endian, merge the two u8 values into a u16 value.
/// For some reason, the orders are reversed?
/// TODO: Figure out Why?
pub fn bytes_to_addr(first: u8, second: u8) -> u16 {
    ((second as u16) << 8) | first as u16
}

pub fn addr_to_bytes(addr: u16) -> (u8, u8) {
    (addr as u8, (addr >> 8) as u8)
}

/// If the src address is on a different page to dest address.
fn is_on_different_pages(src: u16, dest: u16) -> bool {
    src / PAGE_SIZE != dest / PAGE_SIZE
}

enum Register {
    A,
    X,
    Y,
}

impl ToString for Register {
    fn to_string(&self) -> String {
        match self {
            Register::A => "A",
            Register::X => "X",
            Register::Y => "Y",
        }
        .to_string()
    }
}
