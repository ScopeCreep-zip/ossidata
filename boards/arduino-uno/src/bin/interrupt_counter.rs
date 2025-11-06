//! External interrupt counter example
//!
//! This example counts button presses using external interrupts.
//!
//! Hardware setup:
//! - Connect a button between D2 and GND
//! - Internal pull-up resistor is enabled
//! - Built-in LED on D13 will toggle with each press
//! - Serial monitor at 9600 baud shows count
//!
//! The interrupt fires on the FALLING edge (button press to GND).

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Serial, Delay, attach_interrupt, ExternalInterrupt, InterruptMode};
use panic_halt as _;
use core::cell::Cell;
use critical_section::Mutex;

// Counter incremented by interrupt (protected by Mutex for thread safety)
static COUNTER: Mutex<Cell<u16>> = Mutex::new(Cell::new(0));

// Interrupt handler - called when button is pressed
fn button_pressed() {
    // Simply increment the counter
    // Debouncing will be handled in the main loop
    critical_section::with(|cs| {
        let count = COUNTER.borrow(cs).get();
        COUNTER.borrow(cs).set(count.wrapping_add(1));
    });
}

#[avr_device::entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut serial = Serial::new(9600);
    let mut led = peripherals.pins.d13.into_output();
    let mut delay = Delay::new();

    // Configure D2 as input with pull-up
    let _button = peripherals.pins.d2.into_pull_up_input();

    serial.println("Interrupt Counter");
    serial.println("----------------");
    serial.println("Press button on D2");
    serial.println("");

    // Attach interrupt on D2 (INT0) for falling edge
    attach_interrupt(ExternalInterrupt::Int0, InterruptMode::Falling, button_pressed);

    let mut last_count = 0u16;

    loop {
        // Read counter in critical section
        let count = critical_section::with(|cs| COUNTER.borrow(cs).get());

        // Check if count changed
        if count != last_count {
            last_count = count;

            // Toggle LED
            led.toggle();

            // Print count
            serial.write_str("Count: ");
            print_number(&mut serial, count);
            serial.println("");

            // Simple debounce delay after detecting a change
            delay.delay_ms(200);
        }

        delay.delay_ms(10);
    }
}

// Helper function to print a number
fn print_number(serial: &mut Serial, num: u16) {
    if num == 0 {
        serial.write_byte(b'0');
        return;
    }

    let mut n = num;
    let mut digits = [0u8; 10];
    let mut count = 0;

    while n > 0 {
        digits[count] = (n % 10) as u8;
        n /= 10;
        count += 1;
    }

    for i in (0..count).rev() {
        serial.write_byte(b'0' + digits[i]);
    }
}
