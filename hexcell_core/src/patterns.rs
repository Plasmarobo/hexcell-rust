
use embedded_time::duration::*;
use hexcell_api::display::{Led, LED_COUNT, LedBuffer};
use heapless::Vec;
use crate::hexcore_errors::PatternError;
/// Arbitrary, reduce if necessary
pub const MAX_PATTERN_ELEMENTS: usize = 16;
/// One pattern per led (reduce if necessary)
pub const MAX_PATTERN_COUNT: usize = LED_COUNT;

const OFF: Led = Led { r: 0, g: 0, b: 0 };

/// Encoding of a pattern identifyer
#[derive(Copy, Clone, Default)]
#[repr(u8)]
pub enum PatternId
{
    #[default]
    Solid,
    Blink,
    Fade,
    Heartbeat,
    SOS,
}

/// A single pattern element: a color and duration
#[derive(Copy, Clone, Default)]
pub struct PatternElement
{
    pub pattern: PatternId,
    pub color: Led,
    pub duration: Microseconds<u32>
}

#[derive(Clone, Default)]
/// A sequence of pattern elements
pub struct Pattern {
    data: Vec<PatternElement, MAX_PATTERN_ELEMENTS>
}

impl Pattern
{
    pub fn new() -> Pattern
    {
        Pattern
        {
            data: Vec::<PatternElement, MAX_PATTERN_ELEMENTS>::new()
        }
    }

    pub fn next(&mut self, n: PatternElement) -> Result<(), PatternError>
    {
        match self.data.push(n)
        {
            Ok(()) => Ok(()),
            Err(t) => Err(PatternError::PatternSizeError)
        }
    }
}

pub struct PatternBuilder
{
    buffer: Pattern
}

impl PatternBuilder
{
    pub fn new() -> PatternBuilder
    {
        PatternBuilder { buffer: Pattern::new() }
    }

    pub fn then(mut self, n: PatternElement) -> PatternBuilder
    {
        match self.buffer.next(n)
        {
            Ok(()) => {},
            Err(e) => {
                // TODO: Log/ship error
            }
        }
        self
    }

    pub fn finish(self) -> Pattern
    {
        self.buffer
    }
}

/// A stateful index into a pattern
#[derive(Copy, Clone, Default)]
pub struct PatternCursor
{
    // Position with
    element_index: usize,
    pattern_index: usize,
    // Holds the last color value, for blending purposes
    input_buffer: Led,
    elapsed: u32,
    auto_restart: bool,
    enabled: bool
}

/// Pattern Engine
#[derive(Default)]
pub struct PatternEngine
{
    cursors: [PatternCursor; LED_COUNT],
    patterns: Vec<Pattern, MAX_PATTERN_COUNT>,
    output: LedBuffer,
}

impl PatternEngine
{
    pub fn new() -> PatternEngine
    {
        let mut init = Vec::<Pattern, MAX_PATTERN_COUNT>::new();
        for _ in 0..MAX_PATTERN_COUNT
        {
            init.push(Pattern::default());
        }

        PatternEngine {
            cursors: [PatternCursor::default(); LED_COUNT],
            patterns: init,
            output: LedBuffer::default()
        }
    }

    pub fn start(&mut self)
    {
        for cursor in self.cursors.iter_mut()
        {
            cursor.enabled = true;
            cursor.pattern_index = 0;
            cursor.element_index = 0;
            cursor.input_buffer = OFF;
        }
    }

    pub fn run(&mut self, delta: Microseconds<u32>) -> LedBuffer
    {
        for (index, cursor) in self.cursors.iter_mut().enumerate()
        {
            if cursor.enabled
            {
                cursor.elapsed += delta.integer();
                let pattern: &Pattern = &self.patterns[cursor.pattern_index];
                let current_element: &PatternElement = &pattern.data[cursor.element_index];
                if cursor.elapsed > current_element.duration.integer()
                {
                    cursor.elapsed = current_element.duration.integer();
                }
                match current_element.pattern
                {
                    PatternId::Solid => {
                        // Copy color to output
                        self.output[index] = current_element.color
                    },
                    PatternId::Blink => {
                        // Copy color or OFF to buffer
                        if cursor.elapsed < (current_element.duration.integer() >> 1)
                        {
                            self.output[index] = OFF;
                        }
                        else
                        {
                            self.output[index] = current_element.color
                        }
                    },
                    PatternId::Fade => {
                        self.output[index] = cursor.input_buffer.interpolate(current_element.color, cursor.elapsed, current_element.duration.integer());
                    },
                    PatternId::Heartbeat => {},
                    PatternId::SOS => {},
                    _ => {}
                }

                if cursor.elapsed >= current_element.duration.integer()
                {
                    cursor.element_index += 1;
                    if cursor.element_index >= pattern.data.len()
                    {
                        cursor.element_index = 0;
                        if cursor.auto_restart
                        {
                            cursor.enabled = true;
                        }
                        else
                        {
                            cursor.enabled = false;
                        }
                    }
                    if cursor.enabled
                    {
                        cursor.input_buffer = current_element.color;
                        cursor.elapsed = 0;
                    }
                }
            }
        }
        self.output
    }

    pub fn get_output_buffer(&self) -> LedBuffer
    {
        self.output
    }

    pub fn set_pattern(&mut self, at: usize, pattern: Pattern)
    {
        self.patterns[at] = pattern;
    }

    pub fn set_cursor_to_pattern(&mut self, cursor_idx: usize, pattern_idx: usize, restart: bool) -> Result<(), PatternError>
    {
        if cursor_idx < MAX_PATTERN_COUNT
        {
            if let Some(cursor) = self.cursors.get_mut(cursor_idx)
            {
                if pattern_idx < self.patterns.len()
                {
                    cursor.pattern_index = pattern_idx;
                    cursor.auto_restart = restart;
                    Ok(())
                }
                else {
                    Err(PatternError::InvalidPatternError)
                }
            }
            else
            {
                Err(PatternError::InvalidCursorError)
            }
        }
        else
        {
            Err(PatternError::InvalidCursorError)
        }
    }
}
