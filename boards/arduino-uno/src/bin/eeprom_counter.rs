//! Persistent counter using EEPROM
//!
//! This example demonstrates using EEPROM to store a counter that persists
//! across power cycles. Each time the Arduino resets, the counter increments.
//!
//! Hardware setup:
//! - Connect a button between D2 and GND (optional - for manual increment)
//! - Built-in LED on D13 will blink with each increment
//! - Serial monitor at 9600 baud shows counter value
//!
//! The counter is stored at EEPROM address 0-1 (2 bytes for u16).

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Serial, Delay, Eeprom, attach_interrupt, ExternalInterrupt, InterruptMode};
use panic_halt as _;
use core::cell::Cell;
use critical_section::Mutex;

// EEPROM address for storing the counter (2 bytes)
const COUNTER_ADDRESS: u16 = 0;

// Button press flag (set by interrupt)
static BUTTON_PRESSED: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

// Interrupt handler for button press
fn button_handler() {
    critical_section::with(|cs| {
        BUTTON_PRESSED.borrow(cs).set(true);
    });
}

#[avr_device::entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut serial = Serial::new(9600);
    let mut delay = Delay::new();
    let mut led = peripherals.pins.d13.into_output();
    let eeprom = Eeprom::new();

    // Configure D2 as input with pull-up for button
    let _button = peripherals.pins.d2.into_pull_up_input();

    serial.println("Persistent Counter Demo");
    serial.println("----------------------");
    serial.println("");

    // Read current counter from EEPROM
    let mut counter = read_counter_from_eeprom(&eeprom);

    serial.write_str("Startup count: ");
    print_number(&mut serial, counter);
    serial.println("");
    serial.println("");

    // Increment counter for this boot
    counter = counter.wrapping_add(1);

    // Write new counter value to EEPROM
    write_counter_to_eeprom(&eeprom, counter);

    serial.write_str("New count: ");
    print_number(&mut serial, counter);
    serial.println("");
    serial.println("");

    // Blink LED to show increment
    for _ in 0..3 {
        led.set_high();
        delay.delay_ms(100);
        led.set_low();
        delay.delay_ms(100);
    }

    serial.println("Press button on D2 to increment counter");
    serial.println("(or reset Arduino to increment on startup)");
    serial.println("");

    // Attach interrupt for button
    attach_interrupt(ExternalInterrupt::Int0, InterruptMode::Falling, button_handler);

    loop {
        // Check if button was pressed
        let pressed = critical_section::with(|cs| {
            let was_pressed = BUTTON_PRESSED.borrow(cs).get();
            if was_pressed {
                BUTTON_PRESSED.borrow(cs).set(false);
                true
            } else {
                false
            }
        });

        if pressed {
            // Increment counter
            counter = counter.wrapping_add(1);

            // Write to EEPROM
            write_counter_to_eeprom(&eeprom, counter);

            // Show new count
            serial.write_str("Button pressed! New count: ");
            print_number(&mut serial, counter);
            serial.println("");

            // Blink LED
            led.set_high();
            delay.delay_ms(100);
            led.set_low();

            // Debounce delay
            delay.delay_ms(200);
        }

        delay.delay_ms(50);
    }
}

// Read 16-bit counter from EEPROM
fn read_counter_from_eeprom(eeprom: &Eeprom) -> u16 {
    let low = eeprom.read(COUNTER_ADDRESS).unwrap_or(0);
    let high = eeprom.read(COUNTER_ADDRESS + 1).unwrap_or(0);
    ((high as u16) << 8) | (low as u16)
}

// Write 16-bit counter to EEPROM
fn write_counter_to_eeprom(eeprom: &Eeprom, counter: u16) {
    let low = (counter & 0xFF) as u8;
    let high = ((counter >> 8) & 0xFF) as u8;

    // Use update to only write if value changed (reduces wear)
    eeprom.update(COUNTER_ADDRESS, low);
    eeprom.update(COUNTER_ADDRESS + 1, high);
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
