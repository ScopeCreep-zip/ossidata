//! SPI (Serial Peripheral Interface) implementation for Arduino Uno
//!
//! The ATmega328P has a hardware SPI peripheral using pins:
//! - Digital 10 (SS - Slave Select) - Must be managed manually
//! - Digital 11 (MOSI - Master Out Slave In)
//! - Digital 12 (MISO - Master In Slave Out)
//! - Digital 13 (SCK - Serial Clock)
//!
//! This implementation provides master mode SPI communication with
//! transaction-based API for safe multi-device bus sharing.

use core::ptr::{read_volatile, write_volatile};

// SPI registers
const SPCR: *mut u8 = 0x4C as *mut u8;  // SPI Control Register
const SPSR: *mut u8 = 0x4D as *mut u8;  // SPI Status Register
const SPDR: *mut u8 = 0x4E as *mut u8;  // SPI Data Register

// Port B registers (SPI pins are on PORTB)
const DDRB: *mut u8 = 0x24 as *mut u8;   // Data Direction Register B
const PORTB: *mut u8 = 0x25 as *mut u8;  // Port B Data Register

// SPCR bits
const SPE: u8 = 6;   // SPI Enable
const DORD: u8 = 5;  // Data Order (0=MSB first, 1=LSB first)
const MSTR: u8 = 4;  // Master/Slave Select
// Note: SPIE (7), CPOL (3), CPHA (2), SPR1 (1), SPR0 (0) are calculated in mode/clock methods

// SPSR bits
const SPIF: u8 = 7;  // SPI Interrupt Flag
// Note: SPI2X (0) is calculated in clock method

// Pin definitions (PORTB bit positions)
const SS_BIT: u8 = 2;    // PB2 - Digital 10
const MOSI_BIT: u8 = 3;  // PB3 - Digital 11
const SCK_BIT: u8 = 5;   // PB5 - Digital 13
// Note: MISO_BIT (4) automatically becomes input when SPE is set

/// SPI data order
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BitOrder {
    /// Most Significant Bit first (default)
    MsbFirst,
    /// Least Significant Bit first
    LsbFirst,
}

/// SPI clock mode (CPOL/CPHA combinations)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpiMode {
    /// Mode 0: CPOL=0, CPHA=0 (sample on leading edge, clock idle low)
    Mode0,
    /// Mode 1: CPOL=0, CPHA=1 (sample on trailing edge, clock idle low)
    Mode1,
    /// Mode 2: CPOL=1, CPHA=0 (sample on leading edge, clock idle high)
    Mode2,
    /// Mode 3: CPOL=1, CPHA=1 (sample on trailing edge, clock idle high)
    Mode3,
}

impl SpiMode {
    fn to_bits(self) -> u8 {
        match self {
            SpiMode::Mode0 => 0x00,
            SpiMode::Mode1 => 0x04,
            SpiMode::Mode2 => 0x08,
            SpiMode::Mode3 => 0x0C,
        }
    }
}

/// SPI clock speed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpiClock {
    /// 8 MHz (F_CPU/2)
    Div2,
    /// 4 MHz (F_CPU/4) - Default
    Div4,
    /// 2 MHz (F_CPU/8)
    Div8,
    /// 1 MHz (F_CPU/16)
    Div16,
    /// 500 kHz (F_CPU/32)
    Div32,
    /// 250 kHz (F_CPU/64)
    Div64,
    /// 125 kHz (F_CPU/128)
    Div128,
}

impl SpiClock {
    fn to_bits(self) -> (u8, u8) {
        // Returns (SPCR bits, SPSR bits)
        match self {
            SpiClock::Div2 => (0x00, 0x01),   // SPR=00, SPI2X=1
            SpiClock::Div4 => (0x00, 0x00),   // SPR=00, SPI2X=0
            SpiClock::Div8 => (0x01, 0x01),   // SPR=01, SPI2X=1
            SpiClock::Div16 => (0x01, 0x00),  // SPR=01, SPI2X=0
            SpiClock::Div32 => (0x02, 0x01),  // SPR=10, SPI2X=1
            SpiClock::Div64 => (0x02, 0x00),  // SPR=10, SPI2X=0
            SpiClock::Div128 => (0x03, 0x00), // SPR=11, SPI2X=0
        }
    }
}

/// SPI configuration settings for a device
#[derive(Debug, Clone, Copy)]
pub struct SpiSettings {
    spcr: u8,
    spsr: u8,
}

impl SpiSettings {
    /// Create SPI settings
    ///
    /// # Arguments
    /// * `clock` - Clock speed divider
    /// * `bit_order` - Data bit order (MSB/LSB first)
    /// * `mode` - SPI mode (clock polarity and phase)
    pub fn new(clock: SpiClock, bit_order: BitOrder, mode: SpiMode) -> Self {
        let (clock_bits, spi2x) = clock.to_bits();

        let spcr = (1 << SPE) | (1 << MSTR) |
                   (if bit_order == BitOrder::LsbFirst { 1 << DORD } else { 0 }) |
                   mode.to_bits() |
                   clock_bits;

        let spsr = spi2x;

        SpiSettings { spcr, spsr }
    }
}

impl Default for SpiSettings {
    /// Default settings: 4 MHz, MSB first, Mode 0
    fn default() -> Self {
        Self::new(SpiClock::Div4, BitOrder::MsbFirst, SpiMode::Mode0)
    }
}

/// SPI master controller
pub struct Spi {
    _private: (),
}

impl Spi {
    /// Initialize SPI in master mode
    ///
    /// This configures the SPI pins and enables the SPI peripheral.
    /// Note: SS pin (D10) must be controlled manually by the application.
    pub fn new() -> Self {
        unsafe {
            // Set SS high (deselect) before configuring as output
            let portb = read_volatile(PORTB);
            write_volatile(PORTB, portb | (1 << SS_BIT));

            // Configure pins: SS, MOSI, SCK as outputs
            let ddrb = read_volatile(DDRB);
            write_volatile(DDRB, ddrb | (1 << SS_BIT) | (1 << MOSI_BIT) | (1 << SCK_BIT));

            // Enable SPI in master mode (MISO becomes input automatically)
            write_volatile(SPCR, (1 << SPE) | (1 << MSTR));
        }

        Spi { _private: () }
    }

    /// Begin a transaction with specific settings
    ///
    /// This applies the SPI configuration for a specific device.
    /// After this call, manually assert SS (LOW) for the target device.
    ///
    /// # Example
    /// ```no_run
    /// let settings = SpiSettings::new(SpiClock::Div4, BitOrder::MsbFirst, SpiMode::Mode0);
    /// spi.begin_transaction(settings);
    /// // ... control SS pin LOW ...
    /// // ... perform transfers ...
    /// // ... control SS pin HIGH ...
    /// spi.end_transaction();
    /// ```
    pub fn begin_transaction(&mut self, settings: SpiSettings) {
        // Disable interrupts for atomic register update
        unsafe {
            core::arch::asm!("cli");
            write_volatile(SPCR, settings.spcr);
            write_volatile(SPSR, settings.spsr);
            core::arch::asm!("sei");
        }
    }

    /// End a transaction
    ///
    /// Call this after deselecting the device (SS HIGH).
    pub fn end_transaction(&mut self) {
        // Transaction ended - no state to restore
        // (Arduino doesn't restore SPCR/SPSR, only interrupts)
    }

    /// Transfer a single byte (full-duplex)
    ///
    /// Sends `data` and returns the byte received simultaneously.
    ///
    /// # Arguments
    /// * `data` - Byte to send
    ///
    /// # Returns
    /// The byte received during transmission
    pub fn transfer(&mut self, data: u8) -> u8 {
        unsafe {
            // Write data to initiate transfer
            write_volatile(SPDR, data);

            // Wait for transfer to complete
            while read_volatile(SPSR) & (1 << SPIF) == 0 {}

            // Read received data
            read_volatile(SPDR)
        }
    }

    /// Transfer multiple bytes (full-duplex)
    ///
    /// Sends data from `tx_buffer` and writes received data to `rx_buffer`.
    /// Both buffers must be the same length.
    ///
    /// # Arguments
    /// * `tx_buffer` - Data to send
    /// * `rx_buffer` - Buffer to store received data
    pub fn transfer_bytes(&mut self, tx_buffer: &[u8], rx_buffer: &mut [u8]) {
        assert_eq!(tx_buffer.len(), rx_buffer.len());

        for i in 0..tx_buffer.len() {
            rx_buffer[i] = self.transfer(tx_buffer[i]);
        }
    }

    /// Write multiple bytes (ignoring received data)
    ///
    /// # Arguments
    /// * `buffer` - Data to send
    pub fn write(&mut self, buffer: &[u8]) {
        for &byte in buffer {
            let _ = self.transfer(byte);
        }
    }

    /// Read multiple bytes (sending 0x00 for each byte)
    ///
    /// # Arguments
    /// * `buffer` - Buffer to store received data
    pub fn read(&mut self, buffer: &mut [u8]) {
        for byte in buffer.iter_mut() {
            *byte = self.transfer(0x00);
        }
    }

    /// Disable SPI peripheral
    pub fn end(self) {
        unsafe {
            let spcr = read_volatile(SPCR);
            write_volatile(SPCR, spcr & !(1 << SPE));
        }
    }
}
