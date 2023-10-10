use hexcell_api::{messaging::{Message, MessageBuffer}, hexapi_errors::NetworkError};
use heapless::{spsc::Queue, Vec};
use embedded_error_chain::Error;
use embedded_time::duration::*;
use zerocopy::AsBytes;
use core::cell::RefCell;
use crate::scheduler::{Scheduler, global_scheduler};

use crate::ports::{HardPort, PORT_COUNT};

pub const UID_INVALID: u32 = 0;

pub enum MessageStatus
{
    STATUS_OK = 0,  // Success response
    STATUS_ACK,     // No length
    STATUS_NAK,     // No length
    STATUS_QUERY,   // Response expected
    STATUS_TIMEOUT, // Timed out, may be internal
    STATUS_ERROR,   // Error response
}


pub struct GraphInfo
{
    // Rank is the number of cells in a graph
    rank: u16,

}

#[repr(packed)]
#[derive(Copy, Clone, Default, AsBytes)]
pub struct NetworkId
{
    // The orientation of the "root cell is used"
    x: i16,
    y: i16,
    uid: u32,
}

impl NetworkId
{
    pub fn compute_external_id(self, connected_port: HardPort) -> NetworkId
    {
        // Don't compute a UID (will be supplied by a downstream device, if any)
        let mut id = NetworkId { x: self.x, y: self.y, uid: UID_INVALID };
        match connected_port
        {
            HardPort::PORT_A => { id.y += 1 },
            HardPort::PORT_B => { id.y +=1; id.x += 1 },
            HardPort::PORT_C => { id.x -= 1 },
            HardPort::PORT_D => { id.y += 1; id.x -= 1 },
            HardPort::PORT_E => { id.x += 1 },
            HardPort::PORT_F => { id.y -= 1 },
            _ => {}
        };
        id
    }
}
#[repr(u8)]
pub enum PortState
{
    PORT_DISCONNECTED,
    PORT_IDLE, // Port is ready for rx/tx
    PORT_LOCK, // Port is actively recieving
    PORT_ERROR,
}

pub struct PortInfo
{
    address: NetworkId,
    state: PortState,
}

#[repr(u8)]
pub enum NetworkState
{
    UNINITIALIZED,
    INITIALIZING,
    IDLE,
    BUSY,
    ERROR,
    REROOT,
}

#[repr(u8)]
pub enum NetworkQuery
{
    WHOAMI,
    GETID,
    SETID,
    FORWARD,
    ROUTETO,
    ENUMERATE,
    BROADCAST,
    // Must be last
    INVALID,
}

pub struct NetworkFSM
{
    state: NetworkState,
    selected_port: HardPort,
    broadcast_counter: u8,
    message_builder: MessageBuffer,
    id: NetworkId
}

impl NetworkFSM
{
    fn network_query(&mut self, port: HardPort, query: NetworkQuery) -> Message
    {
        self.message_builder.clear();
        match query
        {
            NetworkQuery::SETID => {
                let nid = self.id.compute_external_id(self.selected_port);
                let buffer = nid.as_bytes();
                for byte in buffer
                {
                    self.message_builder.push(*byte);
                }
            }
            _ => (),
        }
        Message::new(port as u8, MessageStatus::STATUS_OK as u8, &self.message_builder)
    }

    pub fn new(scheduler: &RefCell<Scheduler>, id: NetworkId) -> NetworkFSM
    {
        NetworkFSM {
            state: NetworkState::UNINITIALIZED,
            selected_port: HardPort::PORT_A,
            broadcast_counter: 0,
            message_builder: MessageBuffer::new(),
            id
        }
    }

    pub fn schedule(&mut self, status: MessageStatus, )

    pub fn init(&mut self)
    {
        self.update(MessageStatus::STATUS_OK);
        global_scheduler().borrow_mut().queue_task(|| self.update(MessageStatus::STATUS_OK), Microseconds(0), true);
        
    }

    pub fn task_callback(&mut self)
    {
        self.update(MessageStatus::STATUS_OK);
    }

    pub fn task_timeout(&mut self)
    {
        
    }

    pub fn update(&mut self, response: MessagStatus)
    {
        match self.state
        {
            NetworkState::UNINITIALIZED => {
                // Send query via port rank
                let query = self.network_query(self.selected_port, NetworkQuery::WHOAMI);
                
            },
            NetworkState::INITIALIZING => { self.state = NetworkState::IDLE },
            NetworkState::IDLE => {
            },
            NetworkState::BUSY => {},
            NetworkState::ERROR => {},
            NetworkState::REROOT => {},
        }
    }

}

pub struct MessageQueueFSM
{

}
