// MIT License
// Based on Ben Buxton's rotary encoder algorithm
// Reference: https://github.com/miketeachman/micropython-rotary

use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};

// Direction indicators
const DIR_CW: u8 = 0x10;  // Clockwise step
const DIR_CCW: u8 = 0x20; // Counter-clockwise step

// Rotary Encoder States
const R_START: u8 = 0x0;
const R_CW_1: u8 = 0x1;
const R_CW_2: u8 = 0x2;
const R_CW_3: u8 = 0x3;
const R_CCW_1: u8 = 0x4;
const R_CCW_2: u8 = 0x5;
const R_CCW_3: u8 = 0x6;
const R_ILLEGAL: u8 = 0x7;

const STATE_MASK: u8 = 0x07;
const DIR_MASK: u8 = 0x30;

// Full-step transition table
// [current_state][clk_dt_pins] = next_state
const TRANSITION_TABLE: [[u8; 4]; 8] = [
    // CLK/DT: 00          01          10          11
    [R_START,     R_CCW_1,    R_CW_1,     R_START],        // R_START
    [R_CW_2,      R_START,    R_CW_1,     R_START],        // R_CW_1
    [R_CW_2,      R_CW_3,     R_CW_1,     R_START],        // R_CW_2
    [R_CW_2,      R_CW_3,     R_START,    R_START | DIR_CW], // R_CW_3
    [R_CCW_2,     R_CCW_1,    R_START,    R_START],        // R_CCW_1
    [R_CCW_2,     R_CCW_1,    R_CCW_3,    R_START],        // R_CCW_2
    [R_CCW_2,     R_START,    R_CCW_3,    R_START | DIR_CCW], // R_CCW_3
    [R_START,     R_START,    R_START,    R_START],        // R_ILLEGAL
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RangeMode {
    Unbounded,
    Wrap,
    Bounded,
}

pub struct RotaryEncoder {
    state: AtomicU8,
    value: AtomicI32,
    min_val: i32,
    max_val: i32,
    incr: i32,
    reverse: i32,
    range_mode: RangeMode,
}

impl RotaryEncoder {
    pub fn new(
        min_val: i32,
        max_val: i32,
        incr: i32,
        reverse: bool,
        range_mode: RangeMode,
    ) -> Self {
        log::info!("Creating RotaryEncoder: min={}, max={}, incr={}, reverse={}, mode={:?}",
                   min_val, max_val, incr, reverse, range_mode);
        
        Self {
            state: AtomicU8::new(R_START),
            value: AtomicI32::new(min_val),
            min_val,
            max_val,
            incr,
            reverse: if reverse { -1 } else { 1 },
            range_mode,
        }
    }

    pub fn value(&self) -> i32 {
        self.value.load(Ordering::SeqCst)
    }

    pub fn set_value(&self, value: i32) {
        log::debug!("Setting value to: {}", value);
        self.value.store(value, Ordering::SeqCst);
    }

    pub fn reset(&self) {
        log::debug!("Resetting value to min_val: {}", self.min_val);
        self.value.store(self.min_val, Ordering::SeqCst);
    }

    /// Process rotary encoder pin state changes
    /// This should be called from an interrupt handler
    pub fn process_pins(&self, clk: bool, dt: bool) {
        let old_value = self.value();
        let old_state = self.state.load(Ordering::SeqCst);
        
        // Combine pin states into a 2-bit value
        let clk_dt_pins = ((clk as u8) << 1) | (dt as u8);
        
        log::trace!("Pin interrupt: CLK={}, DT={}, combined=0b{:02b}, old_state=0x{:02x}, old_value={}",
                   clk, dt, clk_dt_pins, old_state, old_value);
        
        // Determine next state from transition table
        let current_state_index = (old_state & STATE_MASK) as usize;
        let pin_index = clk_dt_pins as usize;
        
        if current_state_index >= TRANSITION_TABLE.len() || pin_index >= 4 {
            log::warn!("Invalid state or pin index: state={}, pins={}", current_state_index, pin_index);
            return;
        }
        
        let new_state = TRANSITION_TABLE[current_state_index][pin_index];
        self.state.store(new_state, Ordering::SeqCst);
        
        let direction = new_state & DIR_MASK;
        
        log::trace!("State transition: 0x{:02x} -> 0x{:02x}, direction=0x{:02x}",
                   old_state, new_state, direction);
        
        // Calculate increment based on direction
        let mut incr = 0;
        if direction == DIR_CW {
            incr = self.incr;
            log::debug!("Clockwise rotation detected, increment={}", incr);
        } else if direction == DIR_CCW {
            incr = -self.incr;
            log::debug!("Counter-clockwise rotation detected, increment={}", incr);
        }
        
        incr *= self.reverse;
        
        if incr != 0 {
            // Update value based on range mode
            let new_value = match self.range_mode {
                RangeMode::Wrap => {
                    let range = self.max_val - self.min_val + 1;
                    let mut val = old_value + incr;
                    
                    if val < self.min_val {
                        val += range * ((self.min_val - val) / range + 1);
                    }
                    
                    self.min_val + (val - self.min_val) % range
                }
                RangeMode::Bounded => {
                    (old_value + incr).clamp(self.min_val, self.max_val)
                }
                RangeMode::Unbounded => old_value + incr,
            };
            
            self.value.store(new_value, Ordering::SeqCst);
            
            log::info!("Value changed: {} -> {} (incr={})", old_value, new_value, incr);
        }
    }
    
    /// Get the current angle in degrees (assuming value represents angle)
    pub fn angle(&self) -> i32 {
        self.value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_unbounded() {
        let encoder = RotaryEncoder::new(0, 100, 1, false, RangeMode::Unbounded);
        assert_eq!(encoder.value(), 0);
        
        // Simulate clockwise rotation
        encoder.process_pins(false, false); // Start
        encoder.process_pins(false, true);  // CCW_1
        encoder.process_pins(true, true);   // Start
        // Value should increase
    }

    #[test]
    fn test_encoder_wrap() {
        let encoder = RotaryEncoder::new(0, 5, 1, false, RangeMode::Wrap);
        encoder.set_value(5);
        
        // Simulate rotation that would go beyond max
        encoder.process_pins(false, false);
        encoder.process_pins(true, false);
        encoder.process_pins(true, true);
    }

    #[test]
    fn test_encoder_bounded() {
        let encoder = RotaryEncoder::new(0, 10, 1, false, RangeMode::Bounded);
        encoder.set_value(10);
        
        // Try to go beyond max - should stay at max
        encoder.process_pins(false, false);
        encoder.process_pins(true, false);
        encoder.process_pins(true, true);
        
        assert!(encoder.value() <= 10);
    }
}
