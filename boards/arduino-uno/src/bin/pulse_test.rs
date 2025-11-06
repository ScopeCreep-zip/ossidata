//! Pulse measurement test example
//!
//! This example tests the pulse_in function by measuring pulses on a pin.
//! You can use this to test ultrasonic sensors (HC-SR04), RC receivers, or
//! any device that outputs pulses.
//!
//! Hardware setup:
//! - Connect a pulse source to pin 7
//! - For testing, you can use another Arduino pin generating PWM
//! - Serial monitor at 9600 baud shows pulse measurements
//!
//! Simple test: Connect pin 3 (PWM) to pin 7 (pulse input)

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Serial, Delay, pulse_in, PulseState, PwmFrequency};
use panic_halt as _;

const PULSE_PIN: u8 = 7;   // Pin to measure pulses on

#[avr_device::entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut serial = Serial::new(9600);
    let mut delay = Delay::new();

    // Configure pin 7 as input for pulse measurement
    let _pulse_pin = peripherals.pins.d7.into_floating_input();

    // Configure pin 3 for PWM output (for testing)
    let mut test_pin = peripherals.pins.d3.into_output().into_pwm(PwmFrequency::Freq980Hz);

    serial.println("Pulse Measurement Test");
    serial.println("---------------------");
    serial.println("Measuring pulses on pin 7");
    serial.println("Generating test PWM on pin 3");
    serial.println("Connect pin 3 to pin 7 to test");
    serial.println("");

    delay.delay_ms(2000);

    // Start PWM on pin 3 at 50% duty cycle for testing
    test_pin.set_duty(128);

    serial.println("PWM started, beginning measurements...");
    serial.println("");

    loop {
        serial.write_str("Measuring...");

        // Measure HIGH pulse width (in microseconds)
        let high_pulse = pulse_in(PULSE_PIN, PulseState::High, 100000);  // Reduced timeout to 100ms

        serial.write_str(" HIGH: ");
        print_number(&mut serial, high_pulse as u16);

        // Measure LOW pulse width (in microseconds)
        let low_pulse = pulse_in(PULSE_PIN, PulseState::Low, 100000);  // Reduced timeout to 100ms

        serial.write_str(" us, LOW: ");
        print_number(&mut serial, low_pulse as u16);
        serial.write_str(" us");

        if high_pulse > 0 && low_pulse > 0 {
            let total = high_pulse + low_pulse;
            let freq = 1_000_000 / total;
            serial.write_str(", Freq: ~");
            print_number(&mut serial, freq as u16);
            serial.write_str(" Hz");
        }

        serial.println("");
        delay.delay_ms(500);
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
