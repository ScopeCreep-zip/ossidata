//! EEPROM support for Arduino Uno (ATmega328P)
//!
//! The ATmega328P has 1KB (1024 bytes) of EEPROM memory that persists
//! across power cycles. EEPROM has approximately 100,000 write/erase cycles.
//!
//! # Important Notes
//! - EEPROM writes are slow (approximately 3.4ms per byte)
//! - Avoid frequent writes to the same address to prevent wear
//! - Read operations are fast and do not cause wear
//! - Global interrupts are temporarily disabled during write operations
//!
//! Based on information from arduino/ArduinoCore-avr via deepwiki.

use core::ptr::{read_volatile, write_volatile};

// EEPROM registers (ATmega328P)
const EECR: *mut u8 = 0x3F as *mut u8;   // EEPROM Control Register
const EEDR: *mut u8 = 0x40 as *mut u8;   // EEPROM Data Register
const EEARL: *mut u8 = 0x41 as *mut u8;  // EEPROM Address Register Low
const EEARH: *mut u8 = 0x42 as *mut u8;  // EEPROM Address Register High

// EECR bits
const EERE: u8 = 0;   // EEPROM Read Enable
const EEPE: u8 = 1;   // EEPROM Program/Write Enable
const EEMPE: u8 = 2;  // EEPROM Master Program/Write Enable

/// EEPROM size in bytes for ATmega328P
pub const EEPROM_SIZE: u16 = 1024;

/// EEPROM interface
pub struct Eeprom;

impl Eeprom {
    /// Create a new EEPROM interface
    pub fn new() -> Self {
        Self
    }

    /// Read a single byte from EEPROM
    ///
    /// # Arguments
    /// * `address` - EEPROM address (0-1023)
    ///
    /// # Returns
    /// The byte value at the specified address, or None if address is out of range
    ///
    /// # Example
    /// ```no_run
    /// let eeprom = Eeprom::new();
    /// if let Some(value) = eeprom.read(0) {
    ///     // Use value
    /// }
    /// ```
    pub fn read(&self, address: u16) -> Option<u8> {
        if address >= EEPROM_SIZE {
            return None;
        }

        unsafe {
            // Wait for completion of previous write (EEPE bit must be 0)
            while (read_volatile(EECR) & (1 << EEPE)) != 0 {}

            // Set up address register (16-bit address)
            write_volatile(EEARH, (address >> 8) as u8);
            write_volatile(EEARL, address as u8);

            // Start EEPROM read by setting EERE bit
            write_volatile(EECR, read_volatile(EECR) | (1 << EERE));

            // Return data from Data Register
            Some(read_volatile(EEDR))
        }
    }

    /// Write a single byte to EEPROM
    ///
    /// # Arguments
    /// * `address` - EEPROM address (0-1023)
    /// * `value` - Byte value to write
    ///
    /// # Returns
    /// true if successful, false if address is out of range
    ///
    /// # Important
    /// - This operation takes approximately 3.4ms
    /// - Interrupts are temporarily disabled during the write
    /// - Avoid excessive writes to prevent EEPROM wear
    ///
    /// # Example
    /// ```no_run
    /// let eeprom = Eeprom::new();
    /// eeprom.write(0, 42);
    /// ```
    pub fn write(&self, address: u16, value: u8) -> bool {
        if address >= EEPROM_SIZE {
            return false;
        }

        unsafe {
            // Wait for completion of previous write (EEPE bit must be 0)
            while (read_volatile(EECR) & (1 << EEPE)) != 0 {}

            // Set up address and data registers
            write_volatile(EEARH, (address >> 8) as u8);
            write_volatile(EEARL, address as u8);
            write_volatile(EEDR, value);

            // Disable interrupts
            core::arch::asm!("cli");

            // Write logical one to EEMPE (Master Program Enable)
            write_volatile(EECR, read_volatile(EECR) | (1 << EEMPE));

            // Start EEPROM write by setting EEPE (within 4 clock cycles after EEMPE)
            write_volatile(EECR, read_volatile(EECR) | (1 << EEPE));

            // Re-enable interrupts
            core::arch::asm!("sei");
        }

        true
    }

    /// Update a single byte in EEPROM only if it differs from current value
    ///
    /// This reduces EEPROM wear by avoiding unnecessary writes.
    ///
    /// # Arguments
    /// * `address` - EEPROM address (0-1023)
    /// * `value` - Byte value to write
    ///
    /// # Returns
    /// true if successful, false if address is out of range
    pub fn update(&self, address: u16, value: u8) -> bool {
        if address >= EEPROM_SIZE {
            return false;
        }

        // Only write if value is different
        if let Some(current) = self.read(address) {
            if current != value {
                return self.write(address, value);
            }
            true
        } else {
            false
        }
    }

    /// Read a block of bytes from EEPROM
    ///
    /// # Arguments
    /// * `address` - Starting EEPROM address
    /// * `buffer` - Buffer to read data into
    ///
    /// # Returns
    /// Number of bytes successfully read
    ///
    /// # Example
    /// ```no_run
    /// let eeprom = Eeprom::new();
    /// let mut buffer = [0u8; 16];
    /// let bytes_read = eeprom.read_block(0, &mut buffer);
    /// ```
    pub fn read_block(&self, address: u16, buffer: &mut [u8]) -> usize {
        let mut count = 0;
        for i in 0..buffer.len() {
            let addr = address.saturating_add(i as u16);
            if let Some(value) = self.read(addr) {
                buffer[i] = value;
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    /// Write a block of bytes to EEPROM
    ///
    /// # Arguments
    /// * `address` - Starting EEPROM address
    /// * `data` - Data to write
    ///
    /// # Returns
    /// Number of bytes successfully written
    ///
    /// # Important
    /// This operation can take several milliseconds depending on data length.
    /// Each byte takes approximately 3.4ms to write.
    ///
    /// # Example
    /// ```no_run
    /// let eeprom = Eeprom::new();
    /// let data = b"Hello, EEPROM!";
    /// let bytes_written = eeprom.write_block(0, data);
    /// ```
    pub fn write_block(&self, address: u16, data: &[u8]) -> usize {
        let mut count = 0;
        for i in 0..data.len() {
            let addr = address.saturating_add(i as u16);
            if self.write(addr, data[i]) {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    /// Update a block of bytes in EEPROM only if they differ
    ///
    /// This reduces EEPROM wear by avoiding unnecessary writes.
    ///
    /// # Arguments
    /// * `address` - Starting EEPROM address
    /// * `data` - Data to write
    ///
    /// # Returns
    /// Number of bytes successfully updated
    pub fn update_block(&self, address: u16, data: &[u8]) -> usize {
        let mut count = 0;
        for i in 0..data.len() {
            let addr = address.saturating_add(i as u16);
            if self.update(addr, data[i]) {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    /// Clear all EEPROM memory (set to 0xFF)
    ///
    /// # Warning
    /// This operation takes several seconds (approximately 3.5 seconds)
    /// and will wear out the EEPROM. Use sparingly.
    pub fn clear_all(&self) {
        for addr in 0..EEPROM_SIZE {
            self.write(addr, 0xFF);
        }
    }

    /// Check if EEPROM is ready for read/write operations
    ///
    /// # Returns
    /// true if ready, false if a write operation is in progress
    pub fn is_ready(&self) -> bool {
        unsafe { (read_volatile(EECR) & (1 << EEPE)) == 0 }
    }

    /// Wait for EEPROM to be ready
    ///
    /// Blocks until any pending write operation completes
    pub fn wait_ready(&self) {
        unsafe {
            while (read_volatile(EECR) & (1 << EEPE)) != 0 {}
        }
    }

    /// Get the total EEPROM size in bytes
    pub fn size(&self) -> u16 {
        EEPROM_SIZE
    }
}

impl Default for Eeprom {
    fn default() -> Self {
        Self::new()
    }
}
