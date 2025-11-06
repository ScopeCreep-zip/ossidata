//! HD44780 LCD driver for 16x2/20x4 displays with PCF8574 I2C backpack
//!
//! This module provides a driver for HD44780-compatible LCD displays
//! with PCF8574 I2C I/O expander backpack.
//!
//! Hardware connections:
//! - PCF8574 P0 -> LCD RS (Register Select)
//! - PCF8574 P1 -> LCD RW (Read/Write)
//! - PCF8574 P2 -> LCD EN (Enable)
//! - PCF8574 P3 -> LCD Backlight control
//! - PCF8574 P4-P7 -> LCD D4-D7 (4-bit data)
//!
//! Common I2C addresses: 0x27 or 0x3F
//! Use the i2c_scanner example to find your device's address.

use crate::i2c::{I2c, I2cError};
use crate::Delay;

// PCF8574 pin mapping
const RS: u8 = 0x01;  // Register Select (0=command, 1=data)
const EN: u8 = 0x04;  // Enable pulse
const BACKLIGHT: u8 = 0x08;  // Backlight control
// Note: RW (0x02) is not used - always in write mode

// LCD Commands
const LCD_CLEARDISPLAY: u8 = 0x01;
const LCD_RETURNHOME: u8 = 0x02;
const LCD_ENTRYMODESET: u8 = 0x04;
const LCD_DISPLAYCONTROL: u8 = 0x08;
const LCD_FUNCTIONSET: u8 = 0x20;
const LCD_SETDDRAMADDR: u8 = 0x80;

// Entry mode flags
const LCD_ENTRYLEFT: u8 = 0x02;
const LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;

// Display control flags
const LCD_DISPLAYON: u8 = 0x04;
const LCD_CURSORON: u8 = 0x02;
const LCD_CURSOROFF: u8 = 0x00;
const LCD_BLINKON: u8 = 0x01;
const LCD_BLINKOFF: u8 = 0x00;

// Function set flags
const LCD_4BITMODE: u8 = 0x00;
const LCD_2LINE: u8 = 0x08;
const LCD_5X8_DOTS: u8 = 0x00;

// Row offsets for different display sizes
const ROW_OFFSETS: [u8; 4] = [0x00, 0x40, 0x14, 0x54];

/// LCD display controller
pub struct Lcd {
    i2c: I2c,
    address: u8,
    backlight_state: u8,
    delay: Delay,
}

impl Lcd {
    /// Create a new LCD instance
    ///
    /// # Arguments
    /// * `i2c` - I2C peripheral instance
    /// * `address` - I2C address of the LCD (use i2c_scanner to find)
    ///
    /// # Example
    /// ```no_run
    /// let i2c = I2c::new();
    /// let mut lcd = Lcd::new(i2c, 0x3F);  // Use address from i2c_scanner
    /// lcd.init().unwrap();
    /// ```
    pub fn new(i2c: I2c, address: u8) -> Self {
        Lcd {
            i2c,
            address,
            backlight_state: BACKLIGHT,
            delay: Delay::new(),
        }
    }

    /// Initialize the LCD display
    ///
    /// This performs the full initialization sequence as per HD44780 datasheet.
    /// Must be called before using any other LCD functions.
    pub fn init(&mut self) -> Result<(), I2cError> {
        // Wait for LCD power-up
        self.delay.delay_ms(50);

        // Reset I2C expander
        self.i2c_write(self.backlight_state)?;
        self.delay.delay_ms(1000);

        // Start initialization sequence: transition to 4-bit mode
        // Send 0x03 three times with specific delays
        self.write_nibble(0x03 << 4, 0)?;
        self.delay_us(4500);

        self.write_nibble(0x03 << 4, 0)?;
        self.delay_us(4500);

        self.write_nibble(0x03 << 4, 0)?;
        self.delay_us(150);

        // Finally, set to 4-bit interface
        self.write_nibble(0x02 << 4, 0)?;

        // Function set: 4-bit mode, 2 lines, 5x8 dots
        self.send_command(LCD_FUNCTIONSET | LCD_4BITMODE | LCD_2LINE | LCD_5X8_DOTS)?;

        // Display control: display on, cursor off, blink off
        self.send_command(LCD_DISPLAYCONTROL | LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF)?;

        // Clear display
        self.clear()?;

        // Entry mode: left to right, no shift
        self.send_command(LCD_ENTRYMODESET | LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT)?;

        // Home
        self.home()?;

        Ok(())
    }

    /// Write a single byte to the I2C expander
    fn i2c_write(&self, data: u8) -> Result<(), I2cError> {
        self.i2c.write(self.address, &[data])
    }

    /// Pulse the enable pin to latch data
    fn pulse_enable(&mut self, data: u8) -> Result<(), I2cError> {
        // Set EN high
        self.i2c_write(data | EN)?;
        self.delay_us(1);

        // Set EN low
        self.i2c_write(data & !EN)?;
        self.delay_us(50);

        Ok(())
    }

    /// Write a 4-bit nibble to the LCD
    ///
    /// # Arguments
    /// * `nibble` - 4-bit value in upper nibble (bits 4-7)
    /// * `mode` - RS bit (0 for command, RS for data)
    fn write_nibble(&mut self, nibble: u8, mode: u8) -> Result<(), I2cError> {
        let data = nibble | mode | self.backlight_state;
        self.i2c_write(data)?;
        self.pulse_enable(data)?;
        Ok(())
    }

    /// Write a full byte (8 bits) as two nibbles
    ///
    /// # Arguments
    /// * `value` - 8-bit value to write
    /// * `mode` - RS bit (0 for command, RS for data)
    fn write_byte(&mut self, value: u8, mode: u8) -> Result<(), I2cError> {
        // Send high nibble first
        let high_nibble = value & 0xF0;
        self.write_nibble(high_nibble, mode)?;

        // Send low nibble
        let low_nibble = (value << 4) & 0xF0;
        self.write_nibble(low_nibble, mode)?;

        Ok(())
    }

    /// Send a command to the LCD
    fn send_command(&mut self, cmd: u8) -> Result<(), I2cError> {
        self.write_byte(cmd, 0)?;

        // Some commands need extra time
        if cmd == LCD_CLEARDISPLAY || cmd == LCD_RETURNHOME {
            self.delay_us(2000);
        }

        Ok(())
    }

    /// Send data (character) to the LCD
    fn send_data(&mut self, data: u8) -> Result<(), I2cError> {
        self.write_byte(data, RS)
    }

    /// Clear the display
    pub fn clear(&mut self) -> Result<(), I2cError> {
        self.send_command(LCD_CLEARDISPLAY)
    }

    /// Return cursor to home position (0, 0)
    pub fn home(&mut self) -> Result<(), I2cError> {
        self.send_command(LCD_RETURNHOME)
    }

    /// Set cursor position
    ///
    /// # Arguments
    /// * `row` - Row number (0-3, depending on display size)
    /// * `col` - Column number (0-15 for 16x2, 0-19 for 20x4)
    pub fn set_cursor(&mut self, row: u8, col: u8) -> Result<(), I2cError> {
        let row = row.min(3);
        let offset = ROW_OFFSETS[row as usize];
        self.send_command(LCD_SETDDRAMADDR | (col + offset))
    }

    /// Write a single character at the current cursor position
    pub fn write_char(&mut self, ch: char) -> Result<(), I2cError> {
        self.send_data(ch as u8)
    }

    /// Write a string at the current cursor position
    pub fn write_str(&mut self, s: &str) -> Result<(), I2cError> {
        for ch in s.chars() {
            self.write_char(ch)?;
        }
        Ok(())
    }

    /// Write a string at a specific position
    ///
    /// # Arguments
    /// * `row` - Row number (0-3)
    /// * `col` - Column number
    /// * `s` - String to write
    pub fn print_at(&mut self, row: u8, col: u8, s: &str) -> Result<(), I2cError> {
        self.set_cursor(row, col)?;
        self.write_str(s)
    }

    /// Turn backlight on
    pub fn backlight_on(&mut self) -> Result<(), I2cError> {
        self.backlight_state = BACKLIGHT;
        self.i2c_write(self.backlight_state)
    }

    /// Turn backlight off
    pub fn backlight_off(&mut self) -> Result<(), I2cError> {
        self.backlight_state = 0;
        self.i2c_write(self.backlight_state)
    }

    /// Turn display on (with cursor off and blink off)
    pub fn display_on(&mut self) -> Result<(), I2cError> {
        self.send_command(LCD_DISPLAYCONTROL | LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF)
    }

    /// Turn display off
    pub fn display_off(&mut self) -> Result<(), I2cError> {
        self.send_command(LCD_DISPLAYCONTROL)
    }

    /// Show cursor
    pub fn cursor_on(&mut self) -> Result<(), I2cError> {
        self.send_command(LCD_DISPLAYCONTROL | LCD_DISPLAYON | LCD_CURSORON | LCD_BLINKOFF)
    }

    /// Hide cursor
    pub fn cursor_off(&mut self) -> Result<(), I2cError> {
        self.send_command(LCD_DISPLAYCONTROL | LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF)
    }

    /// Enable cursor blink
    pub fn blink_on(&mut self) -> Result<(), I2cError> {
        self.send_command(LCD_DISPLAYCONTROL | LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKON)
    }

    /// Disable cursor blink
    pub fn blink_off(&mut self) -> Result<(), I2cError> {
        self.send_command(LCD_DISPLAYCONTROL | LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF)
    }

    /// Microsecond delay helper
    fn delay_us(&mut self, us: u32) {
        let ms = us / 1000;
        if ms > 0 {
            self.delay.delay_ms(ms);
        }
        // For sub-millisecond delays, use a busy loop
        // At 16MHz: 1us ~= 16 cycles, we use 4 nops per iteration
        let remaining_us = us % 1000;
        for _ in 0..(remaining_us * 4) {
            unsafe { core::arch::asm!("nop"); }
        }
    }
}
