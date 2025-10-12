//! UART/Serial communication implementation for Arduino Uno
//!
//! The ATmega328P has one hardware UART (USART0) connected to pins 0 (RX) and 1 (TX).
//! These pins are also connected to the USB-to-serial converter on the Arduino board.

use core::ptr::{read_volatile, write_volatile};
use ufmt::uWrite;

// USART0 register addresses for ATmega328P
const UDR0: *mut u8 = 0xC6 as *mut u8;   // USART Data Register
const UCSR0A: *mut u8 = 0xC0 as *mut u8; // USART Control and Status Register A
const UCSR0B: *mut u8 = 0xC1 as *mut u8; // USART Control and Status Register B
const UCSR0C: *mut u8 = 0xC2 as *mut u8; // USART Control and Status Register C
const UBRR0L: *mut u8 = 0xC4 as *mut u8; // USART Baud Rate Register Low
const UBRR0H: *mut u8 = 0xC5 as *mut u8; // USART Baud Rate Register High

// UCSR0A bits
const UDRE0: u8 = 5;  // USART Data Register Empty
const RXC0: u8 = 7;   // Receive Complete
const TXC0: u8 = 6;   // Transmit Complete

// UCSR0B bits
const RXEN0: u8 = 4;  // Receiver Enable
const TXEN0: u8 = 3;  // Transmitter Enable

// UCSR0C bits
const UCSZ00: u8 = 1; // Character Size bit 0
const UCSZ01: u8 = 2; // Character Size bit 1

/// Serial port configuration
pub struct Serial {
    // No fields needed - we use global registers
}

impl Serial {
    /// Initialize the serial port with the specified baud rate
    ///
    /// For 16 MHz clock:
    /// - 9600 baud: UBRR = 103
    /// - 115200 baud: UBRR = 8
    /// - 57600 baud: UBRR = 16
    ///
    /// Formula: UBRR = (F_CPU / (16 * BAUD)) - 1
    pub fn new(baud_rate: u32) -> Self {
        unsafe {
            // Calculate UBRR value for 16MHz clock
            let ubrr = match baud_rate {
                9600 => 103u16,
                19200 => 51u16,
                38400 => 25u16,
                57600 => 16u16,
                115200 => 8u16,
                _ => {
                    // Generic formula (may have rounding errors)
                    ((16_000_000u32 / (16 * baud_rate)) - 1) as u16
                }
            };

            // Set baud rate
            write_volatile(UBRR0H, (ubrr >> 8) as u8);
            write_volatile(UBRR0L, (ubrr & 0xFF) as u8);

            // Enable receiver and transmitter
            write_volatile(UCSR0B, (1 << RXEN0) | (1 << TXEN0));

            // Set frame format: 8 data bits, 1 stop bit, no parity
            write_volatile(UCSR0C, (1 << UCSZ01) | (1 << UCSZ00));
        }

        Serial {}
    }

    /// Send a single byte
    pub fn write_byte(&mut self, byte: u8) {
        unsafe {
            // Wait for empty transmit buffer
            while read_volatile(UCSR0A) & (1 << UDRE0) == 0 {}
            // Put data into buffer, sends the data
            write_volatile(UDR0, byte);
        }
    }

    /// Receive a single byte (blocking)
    pub fn read_byte(&mut self) -> u8 {
        unsafe {
            // Wait for data to be received
            while read_volatile(UCSR0A) & (1 << RXC0) == 0 {}
            // Get and return received data from buffer
            read_volatile(UDR0)
        }
    }

    /// Check if data is available to read
    pub fn available(&self) -> bool {
        unsafe {
            read_volatile(UCSR0A) & (1 << RXC0) != 0
        }
    }

    /// Write a string
    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    /// Write a string followed by newline
    pub fn println(&mut self, s: &str) {
        self.write_str(s);
        self.write_byte(b'\r');
        self.write_byte(b'\n');
    }

    /// Print just a newline
    pub fn print_newline(&mut self) {
        self.write_byte(b'\r');
        self.write_byte(b'\n');
    }
}

// Implement uWrite trait for ufmt compatibility
impl uWrite for Serial {
    type Error = core::convert::Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}