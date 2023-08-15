use heapless::spsc::Queue;
use heapless::Vec;

pub const QUEUE_LENGTH: usize = 16;
pub const MESSAGE_SIZE: usize = 128; 

#[derive(Copy, Clone, Default)]
pub struct MessageHeader
{
  pub address: u16,
  pub port: u16,
  pub status: u16,
  pub length: u16,
}

#[derive(Copy, Clone)]
pub struct Message
{
  pub header: MessageHeader,
  pub body: [u8; MESSAGE_SIZE],
}

impl Message
{
  pub fn new(address: u16, port: u16, status: u16, data: &Vec<u8, MESSAGE_SIZE>) -> Message
  {
    let mut m = Message
    {
      header: MessageHeader { address: address, port: port, status: status, length: data.len() as u16 },
      body: [0; MESSAGE_SIZE]
    };
    for (i, dat) in data.iter().enumerate()
    {
      m.body[i] = *dat;
    }
    return m;
  }
}

pub type MessageBuffer = [u8; MESSAGE_SIZE];
pub type MessageQueue = Queue<Message, QUEUE_LENGTH>;