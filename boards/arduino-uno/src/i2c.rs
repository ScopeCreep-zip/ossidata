//! I2C (TWI - Two Wire Interface) implementation for Arduino Uno
//!
//! The ATmega328P has a hardware I2C peripheral using pins:
//! - A4 (SDA - Serial Data)
//! - A5 (SCL - Serial Clock)
//!
//! This implementation provides blocking master mode I2C communication.

use core::ptr::{read_volatile, write_volatile};

// TWI registers
const TWBR: *mut u8 = 0xB8 as *mut u8;   // TWI Bit Rate Register
const TWSR: *mut u8 = 0xB9 as *mut u8;   // TWI Status Register
const TWDR: *mut u8 = 0xBB as *mut u8;   // TWI Data Register
const TWCR: *mut u8 = 0xBC as *mut u8;   // TWI Control Register

// TWCR bits
const TWINT: u8 = 7;  // TWI Interrupt Flag
const TWEA: u8 = 6;   // TWI Enable Acknowledge
const TWSTA: u8 = 5;  // TWI Start Condition
const TWSTO: u8 = 4;  // TWI Stop Condition
const TWEN: u8 = 2;   // TWI Enable
// Note: TWWC (bit 3) and TWIE (bit 0) are not used in basic blocking mode

// TWI Status codes
const TW_START: u8 = 0x08;           // Start condition transmitted
const TW_REP_START: u8 = 0x10;       // Repeated start condition transmitted
const TW_MT_SLA_ACK: u8 = 0x18;      // SLA+W transmitted, ACK received
const TW_MT_SLA_NACK: u8 = 0x20;     // SLA+W transmitted, NACK received
const TW_MT_DATA_ACK: u8 = 0x28;     // Data transmitted, ACK received
const TW_MT_DATA_NACK: u8 = 0x30;    // Data transmitted, NACK received
const TW_MR_SLA_ACK: u8 = 0x40;      // SLA+R transmitted, ACK received
const TW_MR_SLA_NACK: u8 = 0x48;     // SLA+R transmitted, NACK received
const TW_MR_DATA_ACK: u8 = 0x50;     // Data received, ACK returned
const TW_MR_DATA_NACK: u8 = 0x58;    // Data received, NACK returned

const TW_STATUS_MASK: u8 = 0xF8;

// Read/Write bits
const TW_WRITE: u8 = 0;
const TW_READ: u8 = 1;

/// I2C error types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum I2cError {
    /// No acknowledgment received from slave
    Nack,
    /// Timeout waiting for operation
    Timeout,
    /// Bus error or arbitration lost
    BusError,
}

/// I2C master controller
pub struct I2c {
    timeout_us: u32,
}

impl I2c {
    /// Initialize I2C with 100kHz clock (standard mode)
    ///
    /// For 16MHz CPU: TWBR = ((16000000/100000) - 16) / 2 = 72
    pub fn new() -> Self {
        Self::with_frequency(100_000)
    }

    /// Initialize I2C with custom frequency
    ///
    /// Common frequencies:
    /// - 100 kHz (standard mode)
    /// - 400 kHz (fast mode)
    pub fn with_frequency(freq_hz: u32) -> Self {
        unsafe {
            // Calculate TWBR value for desired frequency
            // SCL = CPU_CLK / (16 + 2 * TWBR * prescaler)
            // We use prescaler = 1, so:
            // TWBR = ((CPU_CLK / SCL) - 16) / 2
            let twbr_val = ((16_000_000 / freq_hz) - 16) / 2;
            write_volatile(TWBR, twbr_val as u8);

            // Set prescaler to 1 (TWPS bits = 0)
            write_volatile(TWSR, 0);

            // Enable TWI
            write_volatile(TWCR, 1 << TWEN);
        }

        I2c {
            timeout_us: 10_000, // 10ms default timeout
        }
    }

    /// Set timeout in microseconds
    pub fn set_timeout(&mut self, timeout_us: u32) {
        self.timeout_us = timeout_us;
    }

    /// Wait for TWINT flag with timeout
    fn wait_for_twint(&self) -> Result<(), I2cError> {
        let start = crate::micros();
        unsafe {
            while read_volatile(TWCR) & (1 << TWINT) == 0 {
                if crate::micros().wrapping_sub(start) > self.timeout_us {
                    return Err(I2cError::Timeout);
                }
            }
        }
        Ok(())
    }

    /// Get TWI status
    fn get_status(&self) -> u8 {
        unsafe { read_volatile(TWSR) & TW_STATUS_MASK }
    }

    /// Send START condition
    fn start(&self) -> Result<(), I2cError> {
        unsafe {
            write_volatile(TWCR, (1 << TWINT) | (1 << TWSTA) | (1 << TWEN));
        }
        self.wait_for_twint()?;

        let status = self.get_status();
        if status != TW_START && status != TW_REP_START {
            return Err(I2cError::BusError);
        }
        Ok(())
    }

    /// Send STOP condition
    fn stop(&self) {
        unsafe {
            write_volatile(TWCR, (1 << TWINT) | (1 << TWSTO) | (1 << TWEN));
        }
    }

    /// Write a byte to the bus
    fn write_byte(&self, byte: u8, expected_status: u8) -> Result<(), I2cError> {
        unsafe {
            write_volatile(TWDR, byte);
            write_volatile(TWCR, (1 << TWINT) | (1 << TWEN));
        }
        self.wait_for_twint()?;

        let status = self.get_status();
        if status != expected_status {
            // Check for specific error conditions
            match status {
                TW_MT_SLA_NACK | TW_MT_DATA_NACK | TW_MR_SLA_NACK => Err(I2cError::Nack),
                _ => Err(I2cError::BusError),
            }
        } else {
            Ok(())
        }
    }

    /// Read a byte from the bus
    fn read_byte(&self, send_ack: bool) -> Result<u8, I2cError> {
        unsafe {
            if send_ack {
                write_volatile(TWCR, (1 << TWINT) | (1 << TWEN) | (1 << TWEA));
            } else {
                write_volatile(TWCR, (1 << TWINT) | (1 << TWEN));
            }
        }
        self.wait_for_twint()?;

        let status = self.get_status();
        let expected = if send_ack { TW_MR_DATA_ACK } else { TW_MR_DATA_NACK };
        if status != expected {
            return Err(I2cError::BusError);
        }

        Ok(unsafe { read_volatile(TWDR) })
    }

    /// Write data to an I2C slave device
    ///
    /// # Arguments
    /// * `address` - 7-bit slave address
    /// * `data` - Data bytes to write
    pub fn write(&self, address: u8, data: &[u8]) -> Result<(), I2cError> {
        self.start()?;

        // Send address with write bit
        self.write_byte((address << 1) | TW_WRITE, TW_MT_SLA_ACK)?;

        // Send data bytes
        for &byte in data {
            self.write_byte(byte, TW_MT_DATA_ACK)?;
        }

        self.stop();
        Ok(())
    }

    /// Read data from an I2C slave device
    ///
    /// # Arguments
    /// * `address` - 7-bit slave address
    /// * `buffer` - Buffer to store received data
    pub fn read(&self, address: u8, buffer: &mut [u8]) -> Result<(), I2cError> {
        if buffer.is_empty() {
            return Ok(());
        }

        self.start()?;

        // Send address with read bit
        self.write_byte((address << 1) | TW_READ, TW_MR_SLA_ACK)?;

        // Read all but last byte with ACK
        let last_idx = buffer.len() - 1;
        for i in 0..last_idx {
            buffer[i] = self.read_byte(true)?;
        }

        // Read last byte with NACK
        buffer[last_idx] = self.read_byte(false)?;

        self.stop();
        Ok(())
    }

    /// Write to a register on an I2C device
    ///
    /// # Arguments
    /// * `address` - 7-bit slave address
    /// * `register` - Register address
    /// * `data` - Data bytes to write
    pub fn write_register(&self, address: u8, register: u8, data: &[u8]) -> Result<(), I2cError> {
        self.start()?;

        // Send address with write bit
        self.write_byte((address << 1) | TW_WRITE, TW_MT_SLA_ACK)?;

        // Send register address
        self.write_byte(register, TW_MT_DATA_ACK)?;

        // Send data bytes
        for &byte in data {
            self.write_byte(byte, TW_MT_DATA_ACK)?;
        }

        self.stop();
        Ok(())
    }

    /// Read from a register on an I2C device
    ///
    /// # Arguments
    /// * `address` - 7-bit slave address
    /// * `register` - Register address
    /// * `buffer` - Buffer to store received data
    pub fn read_register(&self, address: u8, register: u8, buffer: &mut [u8]) -> Result<(), I2cError> {
        // Write register address
        self.start()?;
        self.write_byte((address << 1) | TW_WRITE, TW_MT_SLA_ACK)?;
        self.write_byte(register, TW_MT_DATA_ACK)?;

        // Read data with repeated start
        self.read(address, buffer)
    }

    /// Scan the I2C bus for devices
    ///
    /// Returns a list of found device addresses (0-127)
    pub fn scan(&self) -> [bool; 128] {
        let mut found = [false; 128];

        for addr in 0..128u8 {
            // Try to start communication with this address
            if self.start().is_ok() {
                if self.write_byte((addr << 1) | TW_WRITE, TW_MT_SLA_ACK).is_ok() {
                    found[addr as usize] = true;
                }
                self.stop();
            }
        }

        found
    }
}
