use embedded_error_chain::prelude::*;
use embedded_time::duration::*;

use crate::messaging::Message;
use crate::display::LedBuffer;
use crate::hexapi_errors::{NetworkError, PhyError};

pub trait HexCell
{
  // Writes buffer to leds (via SPI or other interface)
  fn update_display(&mut self, buffer: &LedBuffer);
  // Sends a signal
  fn send_signal(&mut self, id: u8, value: u8) -> Result<(), PhyError>;
  // Called when a signal is received
  fn signal_handler(&mut self, id: u8, value: u8);
  // Called when a disconnect event occurs
  fn port_connect_handler(&mut self, port: u8);
  // Called when a connect event occurs
  fn port_disconnect_handler(&mut self, port: u8);
  // Sets the address of this device
  fn set_address(&mut self, address: u32) -> i16;
  // Gets the address of this device
  fn get_address(&self) -> u32;
  // Gets the unique id of this device
  fn get_uid(&self) -> u32;
  // Attempts to dequeue a message from RX queue
  fn get_message(&mut self) -> Option<Message>;
  // Attempts to send a message to a specified address
  fn send_message(&mut self, msg:&Message) -> Result<(), Error<NetworkError>>;
  // Pumps main logic (scheduler while loop, etc)
  fn update(&mut self, now: Microseconds<u32>);
}
