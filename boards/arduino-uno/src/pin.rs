//! Type-safe pin implementation for Arduino Uno
//!
//! This module provides the concrete implementation of pins that connects
//! the type-safe abstractions to actual hardware registers.

use core::marker::PhantomData;
use crate::gpio_impl;

/// Pin modes
pub mod mode {
    /// Input mode marker
    pub struct Input;

    /// Output mode marker
    pub struct Output;

    /// Floating input (no pull resistor)
    pub struct Floating;

    /// Input with pull-up resistor
    pub struct PullUp;
}

/// Hardware pin implementation for Arduino Uno
pub struct Pin<const N: u8, MODE> {
    _mode: PhantomData<MODE>,
}

impl<const N: u8, MODE> Pin<N, MODE> {
    /// Create a new pin
    ///
    /// # Safety
    /// Caller must ensure exclusive access to this pin
    pub const unsafe fn new() -> Self {
        Self {
            _mode: PhantomData,
        }
    }
}

impl<const N: u8> Pin<N, mode::Input> {
    /// Convert to output mode
    pub fn into_output(self) -> Pin<N, mode::Output> {
        unsafe {
            gpio_impl::set_pin_output(N);
            Pin::new()
        }
    }

    /// Convert to floating input
    pub fn into_floating_input(self) -> Pin<N, mode::Floating> {
        unsafe {
            gpio_impl::set_pin_input(N);
            Pin::new()
        }
    }

    /// Convert to pull-up input
    pub fn into_pull_up_input(self) -> Pin<N, mode::PullUp> {
        unsafe {
            gpio_impl::enable_pull_up(N);
            Pin::new()
        }
    }
}

impl<const N: u8> Pin<N, mode::Output> {
    /// Set pin high
    pub fn set_high(&mut self) {
        unsafe {
            gpio_impl::set_pin_high(N);
        }
    }

    /// Set pin low
    pub fn set_low(&mut self) {
        unsafe {
            gpio_impl::set_pin_low(N);
        }
    }

    /// Toggle pin state
    pub fn toggle(&mut self) {
        unsafe {
            gpio_impl::toggle_pin(N);
        }
    }

    /// Convert to input mode
    pub fn into_input(self) -> Pin<N, mode::Input> {
        unsafe {
            gpio_impl::set_pin_input(N);
            Pin::new()
        }
    }
}

impl<const N: u8> Pin<N, mode::Floating> {
    /// Read pin state
    pub fn is_high(&self) -> bool {
        unsafe { gpio_impl::read_pin(N) }
    }

    /// Read pin state (inverted)
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Convert to output mode
    pub fn into_output(self) -> Pin<N, mode::Output> {
        unsafe {
            gpio_impl::set_pin_output(N);
            Pin::new()
        }
    }

    /// Convert to pull-up input
    pub fn into_pull_up_input(self) -> Pin<N, mode::PullUp> {
        unsafe {
            gpio_impl::enable_pull_up(N);
            Pin::new()
        }
    }
}

impl<const N: u8> Pin<N, mode::PullUp> {
    /// Read pin state
    pub fn is_high(&self) -> bool {
        unsafe { gpio_impl::read_pin(N) }
    }

    /// Read pin state (inverted)
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Convert to output mode
    pub fn into_output(self) -> Pin<N, mode::Output> {
        unsafe {
            gpio_impl::set_pin_output(N);
            Pin::new()
        }
    }

    /// Convert to floating input
    pub fn into_floating_input(self) -> Pin<N, mode::Floating> {
        unsafe {
            gpio_impl::set_pin_input(N);
            // Clear the pull-up by ensuring PORT bit is low
            gpio_impl::set_pin_low(N);
            Pin::new()
        }
    }
}

// Arduino-style helper functions for use with pulse and shift functions

/// Pin state values
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PinState {
    /// Low (0V)
    Low,
    /// High (5V)
    High,
}

/// Write a digital value to a pin (Arduino-style digitalWrite)
///
/// # Arguments
/// * `pin` - The pin number (0-13)
/// * `state` - The state to write (High or Low)
pub fn digital_write(pin: u8, state: PinState) {
    if pin > 13 {
        return;
    }

    unsafe {
        match state {
            PinState::High => gpio_impl::set_pin_high(pin),
            PinState::Low => gpio_impl::set_pin_low(pin),
        }
    }
}

/// Read a digital value from a pin (Arduino-style digitalRead)
///
/// # Arguments
/// * `pin` - The pin number (0-13)
///
/// # Returns
/// The current pin state (High or Low)
pub fn digital_read(pin: u8) -> PinState {
    if pin > 13 {
        return PinState::Low;
    }

    unsafe {
        if gpio_impl::read_pin(pin) {
            PinState::High
        } else {
            PinState::Low
        }
    }
}