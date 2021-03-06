use crate::cpu::Cpu;
use crate::opcode::*;

/// AddRegister to be used with AddressMode.
pub enum AddRegister {
    // Don't add any registers
    None,

    // Add X register.
    X,

    // Add Y register.
    Y,
}

pub enum AddressMode {
    _Accumulate,

    Immediate {
        value: u8,
    },

    Relative {
        /// Signed offset.
        offset: i8,
    },

    /// Address the zero page but add the register X.
    ///
    /// @warning We can wrap around, but still can only access first 256 bytes of memory (e.g.
    /// always truncate)!
    ZeroPage {
        register: AddRegister,
        offset: u8,
    },

    /// Index absolutely using full 16 bit address.
    Absolute {
        register: AddRegister,
        address: u16,
    },

    /// Access the location to extract the real jump location from.
    Indirect {
        register: AddRegister,
        address_to_read_indirect: u16,
    },
}

impl AddressMode {
    /// Offset into memory to lookup.
    pub fn to_addr(&self, cpu: &Cpu) -> Option<u16> {
        match &self {
            AddressMode::Relative { offset } => {
                Some((cpu.program_counter as i64 + *offset as i64) as u16)
            }
            AddressMode::ZeroPage { register, offset } => match register {
                AddRegister::None => Some(*offset as u16),

                // Intentionally wrap over.
                AddRegister::X => Some((cpu.x + *offset) as u16),
                AddRegister::Y => Some((cpu.y + *offset) as u16),
            },
            AddressMode::Absolute { register, address } => match register {
                AddRegister::None => Some(*address),

                // Intentionally wrap over.
                AddRegister::X => Some((cpu.x as u16 + *address) as u16),
                AddRegister::Y => Some((cpu.y as u16 + *address) as u16),
            },
            AddressMode::Indirect {
                register: _,
                address_to_read_indirect: _,
            } => {
                panic!("Unsupported!");
            }
            _ => None,
        }
    }

    pub fn to_value(&self, cpu: &Cpu) -> u8 {
        match &self {
            AddressMode::_Accumulate => cpu.a,
            AddressMode::Immediate { value } => *value,
            _ => cpu.memory[self.to_addr(cpu).unwrap() as usize],
        }
    }

    /// Convert the address mode to a string.
    pub fn to_string(&self, cpu: &Cpu) -> String {
        match &self {
            AddressMode::_Accumulate => return "A".to_string(),
            AddressMode::Immediate { value } => return format!("#${:02X}", value),
            _ => (),
        };

        let addr = self.to_addr(cpu).unwrap();
        let value = cpu.memory[addr as usize];
        match &self {
            AddressMode::Relative { offset: _ } => format!("${:04X}", addr),
            AddressMode::ZeroPage {
                register: _,
                offset: _,
            } => {
                // Addr is the zero page address so we only log 2 hex digits.
                format!("${:02X} = {:02X}", addr as u8, value)
            }
            AddressMode::Absolute {
                register: _,
                address: _,
            } => {
                format!("${:04X}", addr)
            }
            _ => panic!("Unsupported!"),
        }
    }

    pub fn value_to_string(&self) -> String {
        match &self {
            AddressMode::_Accumulate => "A".to_string(),
            AddressMode::Relative { offset: value } => {
                format!("{:02X}", *value as u8)
            }
            AddressMode::Immediate { value } => {
                format!("{:02X}", value)
            }
            AddressMode::ZeroPage {
                register: _,
                offset,
            } => {
                // Addr is the zero page address so we only log 2 hex digits.
                format!("{:02X}", offset)
            }
            AddressMode::Absolute {
                register: _,
                address,
            } => {
                format!(
                    "{:02X} {:02X}",
                    addr_to_bytes(*address).0,
                    addr_to_bytes(*address).1
                )
            }
            _ => panic!("Unsupported!"),
        }
    }
}
