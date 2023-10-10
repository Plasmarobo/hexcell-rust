use heapless::spsc::Queue;
use heapless::Vec;

pub const QUEUE_LENGTH: usize = 16;
pub const MESSAGE_SIZE: usize = 256;

pub type MessageBuffer = Vec<u8, MESSAGE_SIZE>;
pub type MessageQueue = Queue<Message, QUEUE_LENGTH>;

#[derive(Copy, Clone, Default)]
pub struct MessageHeader
{
  pub port: u8,
  pub status: u8,
  pub length: u16,
}

#[derive(Clone)]
pub struct Message
{
  pub header: MessageHeader,
  pub body: MessageBuffer,
}

impl Message
{
  pub fn new(port: u8, status: u8, data: &MessageBuffer) -> Message
  {
    Message
    {
      header: MessageHeader { port: port, status: status, length: data.len() as u16 },
      body: data.clone()
    }
  }
}

