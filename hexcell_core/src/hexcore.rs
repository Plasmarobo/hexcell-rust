
use embedded_time::duration::*;
use hexcell_api::display::LedBuffer;
use crate::{patterns::PatternEngine, networking::{NetworkFSM, NetworkId}, scheduler::Scheduler};
use core::cell::RefCell;

// Hex Cell Core logic
pub struct HexCellCore
{
    network: NetworkFSM,
    pattern_engine: PatternEngine,
    last_tick: Microseconds<u32>,
}

impl HexCellCore
{
    pub fn new(scheduler: &RefCell<Scheduler>) -> HexCellCore
    {

        let id = NetworkId::default(); // TODO: read from eeprom
        HexCellCore {
            network: NetworkFSM::new(scheduler, id),
            pattern_engine: PatternEngine::new(),
            last_tick: Microseconds(0),
        }
    }

    pub fn init(&mut self, now: Microseconds<u32>)
    {
        self.network.init();
        self.last_tick = now;
    }

    pub fn tick(&mut self, now: Microseconds<u32>)
    {
        let delta = now - self.last_tick;
        self.network.update();
        self.pattern_engine.run(delta);
        self.last_tick = now;
    }

    pub fn pattern_buffer(&self) -> LedBuffer
    {
        self.pattern_engine.get_output_buffer()
    }
}
