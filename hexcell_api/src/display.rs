use core::ops::{Add, Sub, Div, Mul, AddAssign, SubAssign};

use crate::hexcell::HexCell;
use crate::logging::{write_log, LogLevel, log, LogMessage};
pub const LED_COUNT: usize = 9;

pub type LedBuffer = [Led; LED_COUNT];

#[derive(Copy, Clone, Default, Debug)]
pub struct Led
{
  pub r: u8,
  pub g: u8,
  pub b: u8
}

impl Led
{
  pub fn scale(&self, factor: u8) -> Led
  {
    let r_int = (self.r as u16 * factor as u16);
    let g_int = (self.g as u16 * factor as u16);
    let b_int = (self.b as u16 * factor as u16);
    Led {
      r: (r_int >> 8) as u8,
      g: (g_int >> 8) as u8,
      b: (b_int >> 8) as u8,
    }
  }

  pub fn interpolate(&mut self, target: Led, elapsed: u32, duration: u32) -> Led
  {
    // Factor here is an 8 bit fixed point on the order of 0..1
    let factor: u8 = if elapsed < duration { ((elapsed << 8) / duration) as u8 } else { 0xFF };
    let inverse_factor = (0xFF) - factor;
    let result = self.scale(inverse_factor) + target.scale(factor);
    result
  }

  pub fn div(self, rhs: u8) -> Led
  {
    Led {
      r: self.r / rhs,
      g: self.g / rhs,
      b: self.b / rhs,
    }
  }

  pub fn mul(self, rhs: u8) -> Led
  {
    Led {
      r: self.r * rhs,
      g: self.g * rhs,
      b: self.b * rhs,
    }
  }
}

impl Add for Led
{
  type Output = Led;
  fn add(self, rhs: Led) -> Led {
    Led {
      r: self.r + rhs.r,
      g: self.g + rhs.g,
      b: self.b + rhs.b
    }
  }
}

impl Add for &Led
{
  type Output = <Led as Add>::Output;
  fn add(self, rhs: &Led) -> Self::Output
  {
    Led::add(*self, *rhs)
  }
}

impl AddAssign for Led
{
  fn add_assign(&mut self, rhs: Self) {
    self.r += rhs.r;
    self.g += rhs.g;
    self.b += rhs.b;
  }
}

impl Mul for Led
{
  type Output = Led;
  fn mul(self, rhs: Led) -> Led
  {
    Led 
    {
      r: (self.r * rhs.r) as u8,
      g: (self.g * rhs.g) as u8,
      b: (self.b * rhs.b) as u8,
    }
  }
}

impl Div for Led
{
  type Output = Led;
  fn div(self, rhs: Led) -> Led
  {
    Led
    {
      r: (self.r / rhs.r) as u8,
      g: (self.g / rhs.g) as u8,
      b: (self.b / rhs.b) as u8,
    }
  }
}

impl Sub for Led
{
  type Output = Led;
  fn sub(self, rhs: Self) -> Self::Output {
      Led {
        r: (self.r as i16 - rhs.r as i16) as u8,
        g: (self.g as i16 - rhs.g as i16) as u8,
        b: (self.b as i16 - rhs.b as i16) as u8,
      }
  }
}

impl SubAssign for Led
{
  fn sub_assign(&mut self, rhs: Self) {
      self.r -= rhs.r;
      self.g -= rhs.g;
      self.b -= rhs.b;
  }
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
