//! Arduino Uno board support
//!
//! This crate provides Ossidata support for the Arduino Uno (ATmega328P).

#![no_std]
#![feature(asm_experimental_arch)]

pub use ossidata_core::prelude::*;
use core::mem::MaybeUninit;

// Hardware-specific implementations
mod gpio_impl;
mod pin;
mod serial;

// Re-export our hardware types
pub use pin::Pin;
pub use serial::Serial;

// Critical section implementation is provided by avr-device crate
// with the "critical-section-impl" feature
// We need to ensure avr-device is actually linked
extern crate avr_device;

/// Arduino Uno pin definitions
pub struct Pins {
    /// Digital pin 0 (RX)
    pub d0: Pin<0, pin::mode::Input>,
    /// Digital pin 1 (TX)
    pub d1: Pin<1, pin::mode::Input>,
    /// Digital pin 2
    pub d2: Pin<2, pin::mode::Input>,
    /// Digital pin 3 (PWM)
    pub d3: Pin<3, pin::mode::Input>,
    /// Digital pin 4
    pub d4: Pin<4, pin::mode::Input>,
    /// Digital pin 5 (PWM)
    pub d5: Pin<5, pin::mode::Input>,
    /// Digital pin 6 (PWM)
    pub d6: Pin<6, pin::mode::Input>,
    /// Digital pin 7
    pub d7: Pin<7, pin::mode::Input>,
    /// Digital pin 8
    pub d8: Pin<8, pin::mode::Input>,
    /// Digital pin 9 (PWM)
    pub d9: Pin<9, pin::mode::Input>,
    /// Digital pin 10 (PWM/SS)
    pub d10: Pin<10, pin::mode::Input>,
    /// Digital pin 11 (PWM/MOSI)
    pub d11: Pin<11, pin::mode::Input>,
    /// Digital pin 12 (MISO)
    pub d12: Pin<12, pin::mode::Input>,
    /// Digital pin 13 (LED/SCK)
    pub d13: Pin<13, pin::mode::Input>,
}

impl Pins {
    /// Create a new Pins instance
    ///
    /// # Safety
    /// This should only be called once to ensure exclusive access to pins
    pub unsafe fn new() -> Self {
        Self {
            d0: Pin::new(),
            d1: Pin::new(),
            d2: Pin::new(),
            d3: Pin::new(),
            d4: Pin::new(),
            d5: Pin::new(),
            d6: Pin::new(),
            d7: Pin::new(),
            d8: Pin::new(),
            d9: Pin::new(),
            d10: Pin::new(),
            d11: Pin::new(),
            d12: Pin::new(),
            d13: Pin::new(),
        }
    }
}

/// Peripherals singleton
pub struct Peripherals {
    /// GPIO pins
    pub pins: Pins,
}

static mut PERIPHERALS: MaybeUninit<Peripherals> = MaybeUninit::uninit();
static mut TAKEN: bool = false;

impl Peripherals {
    /// Take the peripherals singleton
    ///
    /// Returns `Some(Peripherals)` the first time, `None` after
    pub fn take() -> Option<Self> {
        critical_section::with(|_| unsafe {
            if TAKEN {
                None
            } else {
                TAKEN = true;
                let peripherals = Peripherals {
                    pins: Pins::new(),
                };
                PERIPHERALS.write(peripherals);
                Some(PERIPHERALS.assume_init_read())
            }
        })
    }
}

/// Delay implementation using busy-wait loops
///
/// For ATmega328P at 16MHz:
/// - 1 clock cycle = 62.5ns
/// - This implementation provides approximate millisecond delays
pub struct Delay;

impl Delay {
    /// Create a new delay instance
    pub fn new() -> Self {
        Self
    }

    /// Delay for the specified number of milliseconds
    ///
    /// At 16MHz, we need approximately 16,000 cycles per millisecond.
    /// This implementation uses a calibrated busy-wait loop.
    pub fn delay_ms(&mut self, ms: u32) {
        // Each iteration of the inner loop takes approximately 4 cycles:
        // - 1 cycle for the loop counter decrement
        // - 1 cycle for the branch
        // - 1 cycle for the NOP
        // - 1 cycle overhead
        //
        // So we need about 4000 iterations per millisecond
        const CYCLES_PER_MS: u32 = 4000;

        for _ in 0..ms {
            for _ in 0..CYCLES_PER_MS {
                unsafe {
                    core::arch::asm!("nop");
                }
            }
        }
    }
}