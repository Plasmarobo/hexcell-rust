// Provides a pure-software model of a hexcell
use embedded_error_chain::prelude::*;
use embedded_time::duration::*;
use embedded_time::{Clock, Instant};
use embedded_time::clock::Error as ClkErr;
use hexcell_core::patterns::{Pattern, PatternElement, PatternBuilder, PatternId};
use std::time as stdtime;
use std::thread;

extern crate hexcell_api;
use hexcell_api::hexcell::HexCell;
use hexcell_api::display::{Led, LedBuffer, Display, LED_COUNT};
use hexcell_api::messaging::Message;
use hexcell_api::hexapi_errors::{PhyError, NetworkError};
use hexcell_api::logging::log;
use piston::RenderArgs;

extern crate hexcell_core;
use hexcell_core::core::HexCellCore;
use hexcell_core::ports::{self, HardPort};

use std::borrow::BorrowMut;
use std::collections::{VecDeque, HashMap, HashSet};
use std::cell::{RefCell, RefMut};

use spmc::{Receiver, Sender};

use crate::renderer::Renderer;

#[derive(Clone, Copy)]
pub enum SimError {
  ExistingDeviceAtCoordinate,
  InvalidConnection,
  ConnectionExists,
  UnknownDevice,

}

pub struct HexCellPort
{
  tx: Option<Sender<Message>>,
  rx: Option<Receiver<Message>>
}

struct HexSimClock{
  start: stdtime::Instant
}
impl HexSimClock
{
  fn new() -> HexSimClock
  {
    HexSimClock { start: stdtime::Instant::now() }
  }
}
impl Clock for HexSimClock
{
  type T = u64;
  const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000_000);
  fn try_now(&self) -> Result<Instant<Self>, ClkErr> {
    let now_instant = stdtime::Instant::now();
    let elapsed: u64 = now_instant.duration_since(self.start).as_micros() as u64;
    Ok(Instant::new(elapsed as Self::T))
  }
  
}

pub struct HexCellSim
{
  pub display: Display,
  pub ports: [HexCellPort; HardPort::PORT_COUNT as usize],
  pub connected_flags: u8,
  pub address: u32,
  pub message_queue: VecDeque<Message>,
  pub core: HexCellCore,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Coordinate
{
  pub x: i32,
  pub y: i32
}

pub struct HexCellNetwork
{
  connection_map: HashMap<Coordinate, HashSet<Coordinate>>, // Coordinate to coordinate
  device_map: HashMap<Coordinate, RefCell<HexCellSim>>, // Coordinate to device model
  clk: HexSimClock,
  last_tick: Instant<HexSimClock>
}

impl HexCell for HexCellSim
{
  // Writes buffer to leds (via SPI or other interface)
  fn update_display(&mut self, buffer: &LedBuffer)
  {
    // Copy display data into render buffer
    for item in buffer.into_iter().enumerate()
    {
      let (led, color): (usize, &Led) = item;
      self.display.set_led(led, *color);
    
    }
  }

  fn send_signal(&mut self, _id: u8, _value: u8) -> Result<(), PhyError>
  {
    // ID here might mean PORT in a hardware context...
    // However in hardware vIX, we don't have a signal line
    Err(PhyError::InvalidSignal)
  }

  fn signal_handler(&mut self, _id: u8, _value: u8)
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
    if msg.header.port >= HardPort::PORT_COUNT as u16 {
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

  fn update(&mut self, now: Microseconds<u32>)
  {
      self.core.tick(now);
      let leds = self.core.pattern_buffer();
      self.update_display(&leds);
  }
}

impl HexCellSim
{
  pub fn new() -> HexCellSim
  {
    HexCellSim {
      display: Display::new(),
      ports: array_init::array_init(|_| { HexCellPort { tx: None, rx: None}}), //HardPort::VP_COUNT as usize],
      connected_flags: 0,
      address: 0,
      message_queue: VecDeque::new(),
      core: HexCellCore::new(),
    }
  }

  pub fn default_init(&mut self)
  {
    let chain = PatternBuilder::new()
    .then(PatternElement { // Solid
      pattern: PatternId::Blink,
      color: Led {r: 255, g: 0, b: 0},
      duration: Microseconds(1_000_000)
    })
    .then(PatternElement { // Solid
      pattern: PatternId::Blink,
      color: Led {r: 0, g: 255, b: 0},
      duration: Microseconds(1_000_000)
    })
    .then(PatternElement { // Solid
      pattern: PatternId::Blink,
      color: Led {r: 0, g: 0, b: 255},
      duration: Microseconds(1_000_000)
    })
    .then(PatternElement { // Blue in
      pattern: PatternId::Fade,
      color: Led {r: 0, g: 0, b: 255},
      duration: Microseconds(1_000_000)
    })
    .then(PatternElement { // Fade OUT
      pattern: PatternId::Fade,
      color: Led {r: 0, g: 0, b: 0},
      duration: Microseconds(2_500_000)
    })
    .then(PatternElement { // Green in
      pattern: PatternId::Fade,
      color: Led {r: 0, g: 255, b: 0},
      duration: Microseconds(1_000_000)
    })
    .then(PatternElement { // Fade OUT
      pattern: PatternId::Fade,
      color: Led {r: 0, g: 0, b: 0},
      duration: Microseconds(2_500_000)
    })
    .then(PatternElement { // Red in
      pattern: PatternId::Fade,
      color: Led {r: 255, g: 0, b: 0},
      duration: Microseconds(1_000_000)
    })
    .then(PatternElement { // Fade OUT
      pattern: PatternId::Fade,
      color: Led {r: 0, g: 0, b: 0},
      duration: Microseconds(2_500_000)
    })
    .finish();
    self.core.pattern_engine.start();
    for i in 0..LED_COUNT
    {
      self.core.pattern_engine.set_pattern(i, chain.clone());
      self.core.pattern_engine.set_cursor_to_pattern(i, 0, true).expect("Invalid cursor setting");
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
    if i32::abs_diff(self.x, other.x) > 1 ||
       i32::abs_diff(self.y , other.y) > 1
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
    let clk = HexSimClock::new();
    let now = clk.try_now().unwrap();
    HexCellNetwork { 
      connection_map: HashMap::new(),
      device_map: HashMap::new(),
      clk: clk,
      last_tick: now,
    }
  }

  fn coordinates_to_ports(from: Coordinate, to: Coordinate) -> Result<(HardPort, HardPort), &'static str>
  {
    let mut y = to.y - from.y;
    let mut x = to.x - from.x;
    let matrix = [
      [HardPort::PORT_D, HardPort::PORT_A, HardPort::PORT_B],
      [HardPort::PORT_C, HardPort::PORT_F, HardPort::PORT_E]
    ];
    let reverse_matrix = [
      [HardPort::PORT_E, HardPort::PORT_F, HardPort::PORT_C],
      [HardPort::PORT_B, HardPort::PORT_A, HardPort::PORT_D]
    ];
    
    if y.abs() == 1 && x.abs() <= 1
    {
      if y == -1
      {
         y = 0;
      }
      x += 1;
      return Ok((matrix[y as usize][x as usize], reverse_matrix[y as usize][x as usize]));
    }
    else
    {
      return Err("coordinates not adjacent or overlap");
    }
  }

  fn get_device(&self, at: Coordinate) -> Option<RefMut<'_, HexCellSim>>
  {
    if let Some(cell) =  self.device_map.get(&at)
    {
      match cell.try_borrow_mut()
      {
        Ok(val) => Some(val),
        Err(_) => None
      }
    }
    else {
      None
    }
  }

  fn connect(source_dev: &mut HexCellSim, source_port: HardPort, dest_dev: &mut HexCellSim, dest_port: HardPort) -> Result<(), Error<NetworkError>>
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

  fn disconnect(source_dev: &mut HexCellSim, source_port: HardPort, dest_dev: &mut HexCellSim, dest_port: HardPort)
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
        let dev = RefCell::new(HexCellSim::new());
        dev.borrow_mut().default_init();
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

  fn insert_connection(&mut self, from: Coordinate, to: Coordinate) -> Result<(), &'static str>{
    if let Some(map) = self.connection_map.get_mut(&from) {
      map.insert(to);
    } else {
      let mut set = HashSet::new();
      set.insert(to);
      self.connection_map.insert(from, set);
    }
    
   
    if let (Some(mut source), Some(mut dest)) = (self.get_device(from), self.get_device(to))
    {
      let (source_port, dest_port) = HexCellNetwork::coordinates_to_ports(from, to)?;
      // Get source port
      match HexCellNetwork::connect(source.borrow_mut(), source_port, dest.borrow_mut(), dest_port)
      {
        Ok(()) => return Ok(()),
        Err(error) => return Err(stringify!("Error inserting connection: {:?}", error))
      }
    }
    else
    {
      return Err("unable to fetch devices for insert");
    }
    
  }

  fn erase_connection(&mut self, from: Coordinate, to: Coordinate) -> Result<(), &'static str> {
    if let (Some(mut source), Some (mut dest)) = (self.get_device(from), self.get_device(to))
    {
      let (source_port, dest_port) = HexCellNetwork::coordinates_to_ports(from, to)?;
      HexCellNetwork::disconnect(source.borrow_mut(), source_port, dest.borrow_mut(), dest_port);
      Ok(())
    }
    else {
      return Err("unable to fetch devices for erase");
    }
  }

  pub fn enable_connection(&mut self, from: Coordinate, to: Coordinate) -> Result<(), SimError>
  {
    // Check that devices exist in both locations
    let _from_dev = match self.device_map.get_mut(&from) {
      None => return Err(SimError::InvalidConnection),
      Some(dev) => dev
    };
    let _to_dev = match self.device_map.get_mut(&to) {
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
    self.erase_connection(from, to);
    self.erase_connection(to, from);
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

  pub fn unique_connections(&self) -> HashSet<(Coordinate, Coordinate)>
  {
    let mut working_set: HashSet<(Coordinate, Coordinate)> = HashSet::new();
    for (source, connections) in self.connection_map.iter()
    {
      for dest in connections
      {
        if working_set.contains(&(*source, *dest)) || working_set.contains(&(*dest, *source))
        {
          continue;
        } else {
          working_set.insert((*source, *dest));
        }
      }
    }
    working_set
  }

  pub fn render(&self, r: &mut Renderer, args: &RenderArgs)
  {
    r.draw_grid(args);
    for (key,cell) in self.device_map.iter() {
      r.draw_sim(args, &cell.borrow(), *key);
    }
    r.draw_connections(args, self.unique_connections());
  }

  pub fn update(&mut self)
  {
    // Move data across connections
    let now = self.clk.try_now().unwrap().duration_since_epoch().try_into().unwrap();
    for (_coord, dev) in self.device_map.iter()
    {
      dev.borrow_mut().update(now);
    }
    self.last_tick = self.clk.try_now().unwrap();
    thread::yield_now();
  }
}
