use embedded_time::duration::*;

pub trait Timer32
{
    fn now(&self) -> Microseconds<u32>;
}
