//! GPIO (General Purpose Input/Output) abstractions
//!
//! This module provides type-safe pin abstractions using Rust's type system
//! to enforce correct pin usage at compile time.

use core::marker::PhantomData;
use embedded_hal::digital::{OutputPin, InputPin, ErrorType, PinState};

/// Pin modes - encoded in the type system
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

/// A type-safe pin abstraction
///
/// The pin number N and MODE are encoded in the type system,
/// preventing incorrect usage at compile time.
pub struct Pin<const N: u8, MODE> {
    _mode: PhantomData<MODE>,
}

/// Error type for GPIO operations
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GpioError {
    /// Pin is not available
    PinNotAvailable,
}

impl embedded_hal::digital::Error for GpioError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

impl<const N: u8, MODE> Pin<N, MODE> {
    /// Create a new pin (unsafe because it doesn't check hardware state)
    pub const unsafe fn new() -> Self {
        Self {
            _mode: PhantomData,
        }
    }

    /// Get the pin number
    pub const fn pin_number(&self) -> u8 {
        N
    }
}

/// Input pin implementation
impl<const N: u8> Pin<N, mode::Input> {
    /// Convert this pin to floating input mode
    pub fn into_floating_input(self) -> Pin<N, mode::Floating> {
        // In real implementation, this would configure the hardware
        unsafe { Pin::new() }
    }

    /// Convert this pin to pull-up input mode
    pub fn into_pull_up_input(self) -> Pin<N, mode::PullUp> {
        // In real implementation, this would configure the hardware
        unsafe { Pin::new() }
    }

    /// Convert this pin to output mode
    pub fn into_output(self) -> Pin<N, mode::Output> {
        // In real implementation, this would configure the hardware
        unsafe { Pin::new() }
    }
}

/// Output pin implementation
impl<const N: u8> Pin<N, mode::Output> {
    /// Convert this pin to input mode
    pub fn into_input(self) -> Pin<N, mode::Input> {
        // In real implementation, this would configure the hardware
        unsafe { Pin::new() }
    }

    /// Set the pin high
    pub fn set_high(&mut self) {
        // In real implementation, this would write to hardware registers
        // For now, this is a placeholder
    }

    /// Set the pin low
    pub fn set_low(&mut self) {
        // In real implementation, this would write to hardware registers
        // For now, this is a placeholder
    }

    /// Toggle the pin state
    pub fn toggle(&mut self) {
        // In real implementation, this would toggle hardware registers
        // For now, this is a placeholder
    }
}

// Implement embedded-hal traits
impl<const N: u8> ErrorType for Pin<N, mode::Output> {
    type Error = GpioError;
}

impl<const N: u8> OutputPin for Pin<N, mode::Output> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    fn set_state(&mut self, state: PinState) -> Result<(), Self::Error> {
        match state {
            PinState::Low => self.set_low(),
            PinState::High => self.set_high(),
        }
        Ok(())
    }
}

impl<const N: u8> ErrorType for Pin<N, mode::Floating> {
    type Error = GpioError;
}

impl<const N: u8> InputPin for Pin<N, mode::Floating> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        // Placeholder - would read hardware register
        Ok(false)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        // Placeholder - would read hardware register
        Ok(true)
    }
}

impl<const N: u8> ErrorType for Pin<N, mode::PullUp> {
    type Error = GpioError;
}

impl<const N: u8> InputPin for Pin<N, mode::PullUp> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        // Placeholder - would read hardware register
        Ok(false)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        // Placeholder - would read hardware register
        Ok(true)
    }
}