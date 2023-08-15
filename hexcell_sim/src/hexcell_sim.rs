// Provides a pure-software model of a hexcell
use embedded_error_chain::prelude::*;

extern crate hexcell_api;
use hexcell_api::hexcell::HexCell;
use hexcell_api::display::{LedBuffer, Display};
use hexcell_api::messaging::Message;
use hexcell_api::errors::{PhyError, NetworkError};
use piston::RenderArgs;
use std::collections::btree_map::Iter;
use std::collections::{VecDeque, HashMap, HashSet};
use std::ops::Deref;
use spmc::{Receiver, Sender};

use crate::renderer::Renderer;

#[derive(Clone, Copy)]
pub enum SimError {
  ExistingDeviceAtCoordinate,
  InvalidConnection,
  ConnectionExists,
  UnknownDevice,

}

#[derive(Clone, Copy)]
pub enum VirtualPort {
  A = 0,
  B,
  C,
  D,
  E,
  F,
  VP_COUNT
}

pub struct HexCellPort
{
  tx: Option<Sender<Message>>,
  rx: Option<Receiver<Message>>
}

pub struct HexCellSim
{
  pub display: Display,
  pub ports: [HexCellPort; VirtualPort::VP_COUNT as usize],
  pub connected_flags: u8,
  pub address: u32,
  pub message_queue: VecDeque<Message>
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Coordinate
{
  pub x: u32,
  pub y: u32
}

pub struct HexCellNetwork
{
  connection_map: HashMap<Coordinate, HashSet<Coordinate>>, // Coordinate to coordinate
  device_map: HashMap<Coordinate, HexCellSim> // Coordinate to device model
}

impl HexCell for HexCellSim
{
  // Writes buffer to leds (via SPI or other interface)
  fn update_display(&mut self, buffer: &LedBuffer)
  {
    // Copy display data into render buffer
    
  }

  fn send_signal(&mut self, id: u8, value: u8) -> Result<(), PhyError>
  {
    // ID here might mean PORT in a hardware context...
    // However in hardware vIX, we don't have a signal line
    Err(PhyError::InvalidSignal)
  }

  fn signal_handler(&mut self, id: u8, value: u8)
  {
    // Noop here, vIX doesn't have a signal line
  }

  fn port_connect_handler(&mut self, port: u8)
  {
    self.connected_flags |= 1 << port;
  }

  fn port_disconnect_handler(&mut self, port: u8)
  {
    self.connected_flags &= !(1 << port);
  }

  fn set_address(&mut self, address: u32) -> i16
  {
    self.address = address;
    return 0
  }

  fn get_address(&self) -> u32
  {
    self.address
  }

  fn get_message(&mut self) -> Option<Message>
  {
    return self.message_queue.pop_front()
  }

  fn send_message(&mut self, msg:&Message) -> Result<(), Error<NetworkError>>
  {
    if msg.header.port >= VirtualPort::VP_COUNT as u16 {
      return Err(PhyError::InvalidPort.chain(NetworkError::DestinationUnreachable));
    }
    match &mut self.ports[msg.header.port as usize].tx
    {
      Some(tx) => {
        match tx.send(*msg)
        {
          Ok(_) => (),
          Err(_) => return Err(Error::new(NetworkError::InvalidConfiguration))
        }
      },
      None => return Err(PhyError::NotConnected.chain(NetworkError::DestinationUnreachable))
    }
    Ok(())
  }
}

impl HexCellSim
{
  pub fn new() -> HexCellSim
  {
    HexCellSim {
      display: Display::new(),
      ports: array_init::array_init(|_| { HexCellPort { tx: None, rx: None}}), //VirtualPort::VP_COUNT as usize],
      connected_flags: 0,
      address: 0,
      message_queue: VecDeque::new()
    }
  }
}

impl Coordinate
{
  pub fn adjacent_coordinates(&self) -> [Coordinate; 6]
  {
    // Arbitrarily defined, using odd-even offsets on columns
    // Shifting every other down column by 0.5 simulates
    // hexagonal adjacency. Cording is CW from the top
    [
      Coordinate { x: self.x, y: self.y - 1 }, // Up
      Coordinate { x: self.x + 1, y: self.y - 1},
      Coordinate { x: self.x + 1, y: self.y },
      Coordinate { x: self.x, y: self.y + 1 }, // Down
      Coordinate { x: self.x - 1, y: self.y},
      Coordinate { x: self.x - 1, y: self.y - 1}
    ]
  }

  pub fn is_adjacent(&self, other: &Coordinate) -> bool
  {
    if u32::abs_diff(self.x, other.x) > 1 ||
       u32::abs_diff(self.y , other.y) > 1
    {
      return false;
    }

    if self.x == other.x
    {
      return true;
    }
    else if self.y == other.y ||
              self.y > other.y
    {
      return true;
    }
    else
    {
        return false;
    }
  }
}

impl HexCellNetwork
{
  pub fn new() -> HexCellNetwork
  {
    HexCellNetwork { 
      connection_map: HashMap::new(),
      device_map: HashMap::new(),
    }
  }

  fn connect(source_dev: &mut HexCellSim, source_port: VirtualPort, dest_dev: &mut HexCellSim, dest_port: VirtualPort) -> Result<(), Error<NetworkError>>
  {
    // We cannot connect a port that is already connected
    match &source_dev.ports[source_port as usize].rx
    {
      Some(_) => return Err(PhyError::LocalResourceBusy.chain(NetworkError::InvalidConfiguration)),
      None => ()
    }
    match &dest_dev.ports[dest_port as usize].rx
    {
      Some(_) => return Err(PhyError::RemoteResourceBusy.chain(NetworkError::InvalidConfiguration)),
      None => ()
    }
    // Bind the queue pairs
    let (srctx, srcrx) = spmc::channel();
    let (desttx, destrx) = spmc::channel();
    
    let mut srcport = &mut source_dev.ports[source_port as usize];
    srcport.tx = Some(srctx);
    srcport.rx = Some(destrx);

    let mut destport = &mut dest_dev.ports[dest_port as usize];
    destport.tx = Some(desttx);
    destport.rx = Some(srcrx);

    source_dev.port_connect_handler(source_port as u8);
    dest_dev.port_connect_handler(dest_port as u8);
    Ok(())
  }

  fn disconnect(source_dev: &mut HexCellSim, source_port: VirtualPort, dest_dev: &mut HexCellSim, dest_port: VirtualPort)
  {
    match &source_dev.ports[source_port as usize].rx
    {
      Some(rx) => rx,
      None => return
    };
    match &dest_dev.ports[dest_port as usize].rx
    {
      Some(rx) => rx,
      None => return
    };
    let mut srcport = &mut source_dev.ports[source_port as usize];
    {
      srcport.rx = None;
      srcport.tx = None;
    }
    let mut destport = &mut dest_dev.ports[dest_port as usize];
    {
      destport.tx = None;
      destport.rx = None;
    }

    source_dev.port_disconnect_handler(source_port as u8);
    dest_dev.port_disconnect_handler(dest_port as u8);
  }

  pub fn move_device(&mut self, from: Coordinate, to: Coordinate) -> Result<(), SimError>
  {
    match self.device_map.get_mut(&to) {
      Some(_) => Err(SimError::ExistingDeviceAtCoordinate),
      None => {
         // Disconnect a device, then reconnect it
        for adjacent in from.adjacent_coordinates()
        {
          self.disable_connection(from, adjacent);
        }
        if let Some((_,dev)) = self.device_map.remove_entry(&from)
        {
          self.device_map.insert(to, dev);
        }
        else
        {
          return Err(SimError::UnknownDevice);
        }
        Ok(())
      }
    }
  }

  pub fn new_device(&mut self, coord: Coordinate) -> Result<(), SimError>
  {
    match self.device_map.get(&coord) {
      Some(_) => Err(SimError::ExistingDeviceAtCoordinate),
      None => {
        let dev = HexCellSim::new();
        self.device_map.insert(coord, dev);
        Ok(())
      }
    }
  }

  pub fn remove_device(&mut self, coord: &Coordinate)
  {
    let set = match self.connection_map.get_mut(coord)
    {
      Some(set) => set,
      None => return
    };

    for other_coord in set.clone()
    {
      self.disable_connection(*coord, other_coord)
    }
  }

  pub fn get_device(&mut self, coord: Coordinate) -> Option<&HexCellSim>
  {
    self.device_map.get(&coord)
  }

  fn insert_connection(&mut self, from: Coordinate, to: Coordinate) {
    if let Some(map) = self.connection_map.get_mut(&from) {
      map.insert(to);
    } else {
      let mut set = HashSet::new();
      set.insert(to);
      self.connection_map.insert(from, set);
    }
  }

  pub fn enable_connection(&mut self, from: Coordinate, to: Coordinate) -> Result<(), SimError>
  {
    // Check that devices exist in both locations
    let from_dev = match self.get_device(from) {
      None => return Err(SimError::InvalidConnection),
      Some(dev) => dev
    };
    let to_dev = match self.get_device(to) {
      None => return Err(SimError::InvalidConnection),
      Some(dev) => dev
    };
    match self.connection_map.get(&from) {
      Some(set) => {
        if set.contains(&to) {
          return Err(SimError::ConnectionExists)
        }
      },
      None => ()
    }
    match self.connection_map.get(&to) {
      Some(set) => {
        if set.contains(&from) {
          return Err(SimError::ConnectionExists)
        }
      },
      None => ()
    }
    // Insert bidirectional connection maps
    self.insert_connection(from, to);
    self.insert_connection(to, from);
    Ok(())
  }

  pub fn disable_connection(&mut self, from: Coordinate, to: Coordinate)
  {
    // Lookup other coordinate connection map
    // Remove this coordinate
    if let Some(set) = self.connection_map.get_mut(&from)
    {
      set.remove(&to);
    }
    if let Some(foriegn_set) = self.connection_map.get_mut(&to)
    {
      foriegn_set.remove(&from);
    }
  }

  pub fn connections(&self, at: Coordinate) -> Option<&HashSet<Coordinate>>
  {
    self.connection_map.get(&at)
  }

  pub fn render(&self, r: &mut Renderer, args: &RenderArgs)
  {
    for (key,cell) in self.device_map.iter() {
      const CELL_RADIUS: f64 = 128.0;
      let x:f64 = key.x as f64 * CELL_RADIUS;
      let y:f64 = key.y as f64 * CELL_RADIUS;
      r.draw_sim(args, &cell, x, y)
    }
  }
}
