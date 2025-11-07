//! embedded-hal trait implementations for Arduino Uno
//!
//! This module provides implementations of embedded-hal traits for
//! Arduino Uno hardware, enabling compatibility with the embedded Rust ecosystem.

use embedded_hal::digital;
use crate::pin::{Pin, mode};

// Digital OutputPin trait implementation
impl<const N: u8> digital::OutputPin for Pin<N, mode::Output> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Pin::set_low(self);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Pin::set_high(self);
        Ok(())
    }
}

impl<const N: u8> digital::ErrorType for Pin<N, mode::Output> {
    type Error = core::convert::Infallible;
}

impl<const N: u8> digital::StatefulOutputPin for Pin<N, mode::Output> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        // Read the PORT register to check output state
        Ok(unsafe { crate::gpio_impl::read_pin(N) })
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_set_high()?)
    }

    fn toggle(&mut self) -> Result<(), Self::Error> {
        Pin::toggle(self);
        Ok(())
    }
}

// Digital InputPin trait implementation for Floating input
impl<const N: u8> digital::InputPin for Pin<N, mode::Floating> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(unsafe { crate::gpio_impl::read_pin(N) })
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!unsafe { crate::gpio_impl::read_pin(N) })
    }
}

impl<const N: u8> digital::ErrorType for Pin<N, mode::Floating> {
    type Error = core::convert::Infallible;
}

// Digital InputPin trait implementation for PullUp input
impl<const N: u8> digital::InputPin for Pin<N, mode::PullUp> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(unsafe { crate::gpio_impl::read_pin(N) })
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!unsafe { crate::gpio_impl::read_pin(N) })
    }
}

impl<const N: u8> digital::ErrorType for Pin<N, mode::PullUp> {
    type Error = core::convert::Infallible;
}

// Delay trait implementations
use embedded_hal::delay::DelayNs;

impl DelayNs for crate::Delay {
    fn delay_ns(&mut self, ns: u32) {
        // Convert nanoseconds to microseconds (rounded up)
        let us = (ns + 999) / 1000;
        if us > 0 {
            crate::delay_micros(us as u16);
        }
    }

    fn delay_us(&mut self, us: u32) {
        if us <= 65535 {
            crate::delay_micros(us as u16);
        } else {
            // Split large delays
            let ms = us / 1000;
            let remaining_us = us % 1000;
            self.delay_ms(ms);
            if remaining_us > 0 {
                crate::delay_micros(remaining_us as u16);
            }
        }
    }

    fn delay_ms(&mut self, ms: u32) {
        crate::Delay::delay_ms(self, ms);
    }
}

// NOTE: Serial traits were removed from embedded-hal 1.0
// The serial module was part of embedded-hal 0.2.x but removed in 1.0
// Our Serial type provides Arduino-compatible API directly without embedded-hal traits
