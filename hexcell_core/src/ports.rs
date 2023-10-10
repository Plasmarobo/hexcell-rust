
// Hardware ports
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum HardPort {
  PORT_A = 0,
  PORT_B,
  PORT_C,
  PORT_D,
  PORT_E,
  PORT_F,
  PORT_COUNT
}

pub const PORT_COUNT: usize = HardPort::PORT_COUNT as usize;


// Maps port => rank
pub const PORT_RANKS: [u8; PORT_COUNT] = [
    0, // PORT_A
    4, // PORT_B
    2, // PORT_C
    3, // PORT_D
    1, // PORT_E
    5, // PORT_F
];

// Maps rank => port
pub const RANKED_PORT: [HardPort; PORT_COUNT] = [
    HardPort::PORT_A,
    HardPort::PORT_E,
    HardPort::PORT_C,
    HardPort::PORT_D,
    HardPort::PORT_B,
    HardPort::PORT_F,
];
