//! Simple melody example
//!
//! This example plays "Twinkle Twinkle Little Star" using the tone() function.
//! It demonstrates how to create melodies with notes and rests.
//!
//! Hardware setup:
//! - Connect a piezo buzzer or speaker between pin 11 and GND
//! - A current-limiting resistor (100-330 ohms) is recommended
//! - Serial monitor at 9600 baud shows melody progress
//!
//! The melody repeats continuously.

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Serial, Delay, tone, no_tone};
use panic_halt as _;

// Musical note frequencies (Hz)
const NOTE_C4: u16 = 262;
const NOTE_D4: u16 = 294;
const NOTE_E4: u16 = 330;
const NOTE_F4: u16 = 349;
const NOTE_G4: u16 = 392;
const NOTE_A4: u16 = 440;
const REST: u16 = 0;

const TONE_PIN: u8 = 11;

// Note durations (in milliseconds)
const HALF_NOTE: u32 = 500;
const QUARTER_NOTE: u32 = 250;

// Melody: Twinkle Twinkle Little Star
// Each element is (frequency, duration)
const MELODY: [(u16, u32); 42] = [
    // Twinkle twinkle little star
    (NOTE_C4, QUARTER_NOTE),
    (NOTE_C4, QUARTER_NOTE),
    (NOTE_G4, QUARTER_NOTE),
    (NOTE_G4, QUARTER_NOTE),
    (NOTE_A4, QUARTER_NOTE),
    (NOTE_A4, QUARTER_NOTE),
    (NOTE_G4, HALF_NOTE),

    // How I wonder what you are
    (NOTE_F4, QUARTER_NOTE),
    (NOTE_F4, QUARTER_NOTE),
    (NOTE_E4, QUARTER_NOTE),
    (NOTE_E4, QUARTER_NOTE),
    (NOTE_D4, QUARTER_NOTE),
    (NOTE_D4, QUARTER_NOTE),
    (NOTE_C4, HALF_NOTE),

    // Up above the world so high
    (NOTE_G4, QUARTER_NOTE),
    (NOTE_G4, QUARTER_NOTE),
    (NOTE_F4, QUARTER_NOTE),
    (NOTE_F4, QUARTER_NOTE),
    (NOTE_E4, QUARTER_NOTE),
    (NOTE_E4, QUARTER_NOTE),
    (NOTE_D4, HALF_NOTE),

    // Like a diamond in the sky
    (NOTE_G4, QUARTER_NOTE),
    (NOTE_G4, QUARTER_NOTE),
    (NOTE_F4, QUARTER_NOTE),
    (NOTE_F4, QUARTER_NOTE),
    (NOTE_E4, QUARTER_NOTE),
    (NOTE_E4, QUARTER_NOTE),
    (NOTE_D4, HALF_NOTE),

    // Twinkle twinkle little star
    (NOTE_C4, QUARTER_NOTE),
    (NOTE_C4, QUARTER_NOTE),
    (NOTE_G4, QUARTER_NOTE),
    (NOTE_G4, QUARTER_NOTE),
    (NOTE_A4, QUARTER_NOTE),
    (NOTE_A4, QUARTER_NOTE),
    (NOTE_G4, HALF_NOTE),

    // How I wonder what you are
    (NOTE_F4, QUARTER_NOTE),
    (NOTE_F4, QUARTER_NOTE),
    (NOTE_E4, QUARTER_NOTE),
    (NOTE_E4, QUARTER_NOTE),
    (NOTE_D4, QUARTER_NOTE),
    (NOTE_D4, QUARTER_NOTE),
    (NOTE_C4, HALF_NOTE),
];

#[avr_device::entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut serial = Serial::new(9600);
    let mut delay = Delay::new();

    // Configure pin 11 as output for tone
    let _tone_pin = peripherals.pins.d11.into_output();

    serial.println("Simple Melody Example");
    serial.println("--------------------");
    serial.println("Playing: Twinkle Twinkle Little Star");
    serial.println("Connect buzzer to pin 11");
    serial.println("");

    delay.delay_ms(2000);

    loop {
        serial.println("Starting melody...");

        for &(frequency, duration) in &MELODY {
            if frequency == REST {
                // Rest (silence)
                no_tone(TONE_PIN);
            } else {
                // Play note
                tone(TONE_PIN, frequency);
            }

            // Wait for note duration
            delay.delay_ms(duration);

            // Small pause between notes
            no_tone(TONE_PIN);
            delay.delay_ms(50);
        }

        serial.println("Melody complete!");
        serial.println("");

        // Pause before repeating
        delay.delay_ms(2000);
    }
}
