//! Pulse measurement functions
//!
//! This module provides functions to measure pulse widths on digital pins.
//! It includes both `pulse_in` (optimized) and `pulse_in_long` (using micros()).
//!
//! Based on information from arduino/ArduinoCore-avr via deepwiki.

use crate::time::micros;
use crate::pin::{digital_read, PinState};

/// Pulse state to measure
#[derive(Copy, Clone, PartialEq)]
pub enum PulseState {
    /// Measure HIGH pulse
    High,
    /// Measure LOW pulse
    Low,
}

impl From<PulseState> for PinState {
    fn from(state: PulseState) -> Self {
        match state {
            PulseState::High => PinState::High,
            PulseState::Low => PinState::Low,
        }
    }
}

/// Measures the length of a pulse on a pin (in microseconds)
///
/// This function measures the time that a pulse is in the specified state.
/// Works on pulses from 10 microseconds to 3 minutes in length.
///
/// # Arguments
/// * `pin` - The pin number to measure (0-13)
/// * `state` - The pulse state to measure (High or Low)
/// * `timeout_us` - Timeout in microseconds (default: 1000000 = 1 second)
///
/// # Returns
/// The pulse width in microseconds, or 0 if timeout occurs
///
/// # Example
/// ```no_run
/// use arduino_uno::{pulse_in, PulseState};
///
/// // Measure HIGH pulse on pin 7 with 1 second timeout
/// let pulse_width = pulse_in(7, PulseState::High, 1000000);
/// ```
pub fn pulse_in(pin: u8, state: PulseState, timeout_us: u32) -> u32 {
    // For simplicity, we'll use the pulse_in_long implementation
    // which relies on micros(). A future optimization could add
    // assembly-optimized cycle counting like Arduino does.
    pulse_in_long(pin, state, timeout_us)
}

/// Measures the length of a pulse on a pin using micros() (in microseconds)
///
/// This is a C-based implementation that uses micros() for timing.
/// It can be used in most contexts but may have lower resolution than
/// the assembly-optimized pulse_in for very short pulses.
///
/// # Arguments
/// * `pin` - The pin number to measure (0-13)
/// * `state` - The pulse state to measure (High or Low)
/// * `timeout_us` - Timeout in microseconds
///
/// # Returns
/// The pulse width in microseconds, or 0 if timeout occurs
///
/// # Example
/// ```no_run
/// use arduino_uno::{pulse_in_long, PulseState};
///
/// // Measure LOW pulse on pin 7
/// let pulse_width = pulse_in_long(7, PulseState::Low, 1000000);
/// ```
pub fn pulse_in_long(pin: u8, state: PulseState, timeout_us: u32) -> u32 {
    if pin > 13 {
        return 0;
    }

    let target_state: PinState = state.into();
    let start_micros = micros();

    // Wait for any previous pulse to end
    while digital_read(pin) == target_state {
        if micros().wrapping_sub(start_micros) > timeout_us {
            return 0;
        }
    }

    // Wait for the pulse to start
    while digital_read(pin) != target_state {
        if micros().wrapping_sub(start_micros) > timeout_us {
            return 0;
        }
    }

    // Record when pulse starts
    let pulse_start = micros();

    // Wait for the pulse to stop
    while digital_read(pin) == target_state {
        if micros().wrapping_sub(start_micros) > timeout_us {
            return 0;
        }
    }

    // Calculate and return pulse duration
    micros().wrapping_sub(pulse_start)
}
