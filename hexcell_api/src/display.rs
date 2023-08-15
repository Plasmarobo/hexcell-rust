use crate::hexcell::HexCell;

pub const LED_COUNT: usize = 9;

pub type LedBuffer = [Led; LED_COUNT];

#[derive(Copy, Clone, Default)]
pub struct Led
{
  pub r: u8,
  pub g: u8,
  pub b: u8
}

// This may change, but should be a compile-time constant
pub struct Display
{
  pub leds: LedBuffer
}

impl Display {
  pub fn new() -> Display
  {
    Display {leds: [Led::default(); LED_COUNT]}
  }

  pub fn clear(&mut self)
  {
    self.set_all(Led { r: 0, g: 0, b: 0});
  }

  pub fn set_led(&mut self, idx: usize, value: Led)
  {
    if idx < LED_COUNT
    {
      self.leds[idx] = value;
    }
  }

  pub fn set_all(&mut self, value: Led)
  {
    for led in &mut self.leds
    {
      *led = value;
    }
  }

  pub fn commit<T: HexCell>(&mut self, mut device: T)
  {
    // Call down to device or model
    device.update_display(&self.leds);
  }
}
