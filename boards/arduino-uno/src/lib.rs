//! Arduino Uno board support
//!
//! This crate provides Ossidata support for the Arduino Uno (ATmega328P).

#![no_std]
#![feature(asm_experimental_arch)]
#![feature(abi_avr_interrupt)]

pub use ossidata_core::prelude::*;
use core::mem::MaybeUninit;

// Hardware-specific implementations
mod gpio_impl;
mod gpio;
mod pin;
mod ports;
mod serial;
mod pwm;
mod adc;
mod time;
mod i2c;
mod lcd;
mod spi;
mod rtc;
mod interrupt;
mod eeprom;
mod tone;
mod pulse;
mod shift;
mod watchdog;
mod sleep;
mod embedded_hal_impl;
mod utils;
mod constants;
mod progmem;
mod pcint;
mod timer;
mod memory;
mod software_serial;
mod string;
mod servo;

// Re-export our hardware types
pub use pin::{Pin, PinState, digital_read, digital_write};
pub use gpio::{pin_mode, analog_write, analog_reference};
pub use ports::{
    Port, digital_pin_to_port, digital_pin_to_bit_mask,
    port_output_register, port_input_register, port_mode_register,
    port_write, port_read, port_direction,
    fast_digital_write, fast_digital_read,
};
pub use serial::Serial;
pub use pwm::{Pwm, PwmFrequency};
pub use adc::{Adc, AdcReference};
pub use time::{millis, micros, delay_micros};
pub use i2c::{I2c, I2cError};
pub use lcd::Lcd;
pub use spi::{Spi, SpiSettings, SpiClock, SpiMode, BitOrder};
pub use rtc::{DateTime, Rtc, RtcError, DS1307, DS3231};
pub use interrupt::{attach_interrupt, detach_interrupt, disable_interrupts, restore_interrupts, ExternalInterrupt, InterruptMode};
pub use eeprom::{Eeprom, EEPROM_SIZE};
pub use tone::{tone, tone_duration, no_tone};
pub use pulse::{pulse_in, pulse_in_long, PulseState};
pub use shift::{shift_out, shift_in};
pub use watchdog::{Watchdog, WatchdogTimeout};
pub use sleep::{Sleep, SleepMode};
pub use progmem::{FlashString, pgm_read_byte, pgm_read_word, pgm_read_dword, pgm_read_float, pgm_read_ptr};
pub use pcint::{PcintBank, pcint_attach, pcint_detach, pcint_enable_bank, pcint_disable_bank};
pub use timer::{
    Timer, Prescaler, TimerMode,
    timer_read, timer_write, timer_set_prescaler,
    timer_set_compare_a, timer_set_compare_b,
    timer_enable_overflow_interrupt, timer_disable_overflow_interrupt,
    timer_enable_compare_a_interrupt, timer_disable_compare_a_interrupt,
    timer_enable_compare_b_interrupt, timer_disable_compare_b_interrupt,
    timer1_set_icr, timer_stop, timer_start,
    timer0_set_mode, timer1_set_mode, timer2_set_mode,
    timer_clear_flags, timer1_force_output_compare_a, timer1_force_output_compare_b,
};
pub use memory::{
    free_memory, get_stack_pointer, data_size, bss_size,
    heap_start, heap_end, ram_size, ram_start_address, ram_end_address,
    memory_info, MemoryInfo, check_stack_space,
    fill_memory, count_pattern,
};
pub use software_serial::SoftwareSerial;
pub use string::{ArduinoString, String, DEFAULT_STRING_CAPACITY};
pub use servo::Servo;

// Utility functions
pub use utils::{
    map, constrain, min, max, abs, sq,
    random, random_max, random_seed,
    radians, degrees, round,
    bit, bit_read, bit_set, bit_clear, bit_toggle, bit_write,
    low_byte, high_byte, make_word,
    interrupts, no_interrupts, yield_now,
};

// Arduino-compatible constants
pub use constants::{
    HIGH, LOW,
    INPUT, OUTPUT, INPUT_PULLUP,
    PI, HALF_PI, TWO_PI, EULER, DEG_TO_RAD, RAD_TO_DEG,
    LSBFIRST, MSBFIRST,
    CHANGE, FALLING, RISING,
    DEC, HEX, OCT, BIN,
};

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

                // Initialize Timer0 for millis()/micros() timekeeping
                time::init_timer();

                let peripherals = Peripherals {
                    pins: Pins::new(),
                };
                let ptr = core::ptr::addr_of_mut!(PERIPHERALS);
                (*ptr).write(peripherals);
                Some((*ptr).assume_init_read())
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