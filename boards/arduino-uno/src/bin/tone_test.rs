//! Tone generation test example
//!
//! This example tests the tone() function by playing different frequencies.
//! It cycles through musical notes and demonstrates tone generation.
//!
//! Hardware setup:
//! - Connect a piezo buzzer or speaker between pin 11 and GND
//! - A current-limiting resistor (100-330 ohms) is recommended
//! - Serial monitor at 9600 baud shows test progress
//!
//! The example plays a chromatic scale from C4 to C5.

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Serial, Delay, tone, no_tone};
use panic_halt as _;

// Musical note frequencies (Hz)
const NOTE_C4: u16 = 262;
const NOTE_CS4: u16 = 277;
const NOTE_D4: u16 = 294;
const NOTE_DS4: u16 = 311;
const NOTE_E4: u16 = 330;
const NOTE_F4: u16 = 349;
const NOTE_FS4: u16 = 370;
const NOTE_G4: u16 = 392;
const NOTE_GS4: u16 = 415;
const NOTE_A4: u16 = 440;
const NOTE_AS4: u16 = 466;
const NOTE_B4: u16 = 494;
const NOTE_C5: u16 = 523;

const TONE_PIN: u8 = 11;

#[avr_device::entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut serial = Serial::new(9600);
    let mut delay = Delay::new();

    // Configure pin 11 as output for tone
    let _tone_pin = peripherals.pins.d11.into_output();

    serial.println("Tone Generation Test");
    serial.println("-------------------");
    serial.println("Playing chromatic scale from C4 to C5");
    serial.println("Connect buzzer to pin 11");
    serial.println("");

    delay.delay_ms(1000);

    let notes = [
        (NOTE_C4, "C4"),
        (NOTE_CS4, "C#4"),
        (NOTE_D4, "D4"),
        (NOTE_DS4, "D#4"),
        (NOTE_E4, "E4"),
        (NOTE_F4, "F4"),
        (NOTE_FS4, "F#4"),
        (NOTE_G4, "G4"),
        (NOTE_GS4, "G#4"),
        (NOTE_A4, "A4"),
        (NOTE_AS4, "A#4"),
        (NOTE_B4, "B4"),
        (NOTE_C5, "C5"),
    ];

    loop {
        serial.println("Starting scale...");
        serial.println("");

        for &(frequency, note_name) in &notes {
            // Print note being played
            serial.write_str("Playing ");
            serial.write_str(note_name);
            serial.write_str(" (");
            print_number(&mut serial, frequency);
            serial.write_str(" Hz)");
            serial.println("");

            // Play the tone
            tone(TONE_PIN, frequency);
            delay.delay_ms(500);

            // Stop the tone
            no_tone(TONE_PIN);
            delay.delay_ms(100);
        }

        serial.println("");
        serial.println("Scale complete!");
        serial.println("");

        delay.delay_ms(2000);
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
