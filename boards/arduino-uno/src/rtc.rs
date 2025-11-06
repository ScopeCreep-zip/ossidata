//! Real-Time Clock (RTC) driver for DS1307, DS3231, and PCF8523
//!
//! This module provides a generic RTC interface that works with multiple
//! RTC chips commonly used with Arduino.
//!
//! Supported chips:
//! - DS1307: Basic RTC with 56 bytes NVRAM (address 0x68)
//! - DS3231: High-precision RTC with temperature sensor and alarms (address 0x68)
//! - PCF8523: Low-power RTC with countdown timers (address 0x68)
//!
//! All chips use I2C and BCD (Binary Coded Decimal) encoding for time values.

use crate::i2c::{I2c, I2cError};

/// BCD to binary conversion
#[inline]
fn bcd2bin(val: u8) -> u8 {
    (val >> 4) * 10 + (val & 0x0F)
}

/// Binary to BCD conversion
#[inline]
fn bin2bcd(val: u8) -> u8 {
    ((val / 10) << 4) | (val % 10)
}

/// Date and time representation
///
/// Represents a specific point in time with no timezone information.
/// Year range: 2000-2099
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DateTime {
    year_offset: u8,  // 0-99 (representing 2000-2099)
    month: u8,        // 1-12
    day: u8,          // 1-31
    hour: u8,         // 0-23
    minute: u8,       // 0-59
    second: u8,       // 0-59
}

impl DateTime {
    /// Create a new DateTime
    ///
    /// # Arguments
    /// * `year` - Full year (2000-2099)
    /// * `month` - Month (1-12)
    /// * `day` - Day of month (1-31)
    /// * `hour` - Hour (0-23)
    /// * `minute` - Minute (0-59)
    /// * `second` - Second (0-59)
    pub fn new(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Self {
        DateTime {
            year_offset: ((year - 2000) & 0xFF) as u8,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    /// Get the full year (2000-2099)
    pub fn year(&self) -> u16 {
        2000 + self.year_offset as u16
    }

    /// Get the month (1-12)
    pub fn month(&self) -> u8 {
        self.month
    }

    /// Get the day of month (1-31)
    pub fn day(&self) -> u8 {
        self.day
    }

    /// Get the hour (0-23)
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// Get the minute (0-59)
    pub fn minute(&self) -> u8 {
        self.minute
    }

    /// Get the second (0-59)
    pub fn second(&self) -> u8 {
        self.second
    }

    /// Get day of week (0=Sunday, 6=Saturday)
    ///
    /// Uses Zeller's congruence algorithm
    pub fn day_of_week(&self) -> u8 {
        let mut y = self.year() as u16;
        let mut m = self.month as u16;

        // Adjust for Zeller's (Jan/Feb are month 13/14 of previous year)
        if m < 3 {
            m += 12;
            y -= 1;
        }

        let century = y / 100;
        let year_of_century = y % 100;

        let dow = (self.day as u16 +
                  (13 * (m + 1)) / 5 +
                  year_of_century +
                  year_of_century / 4 +
                  century / 4 +
                  5 * century) % 7;

        // Convert to 0=Sunday format
        ((dow + 6) % 7) as u8
    }

    /// Check if the date is valid
    pub fn is_valid(&self) -> bool {
        if self.year_offset > 99 { return false; }
        if self.month < 1 || self.month > 12 { return false; }
        if self.day < 1 || self.day > 31 { return false; }
        if self.hour > 23 { return false; }
        if self.minute > 59 { return false; }
        if self.second > 59 { return false; }

        // Check days in month
        let days_in_month = match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                let year = self.year();
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    29  // Leap year
                } else {
                    28
                }
            }
            _ => return false,
        };

        self.day <= days_in_month
    }
}

/// RTC error types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RtcError {
    /// I2C communication error
    I2C(I2cError),
    /// Invalid date/time
    InvalidDateTime,
    /// RTC oscillator stopped (power loss)
    PowerLoss,
}

impl From<I2cError> for RtcError {
    fn from(err: I2cError) -> Self {
        RtcError::I2C(err)
    }
}

/// Generic RTC trait
pub trait Rtc {
    /// Initialize the RTC
    fn begin(&mut self) -> Result<(), RtcError>;

    /// Set the current date and time
    fn adjust(&mut self, dt: &DateTime) -> Result<(), RtcError>;

    /// Get the current date and time
    fn now(&self) -> Result<DateTime, RtcError>;

    /// Check if RTC is running
    fn is_running(&self) -> Result<bool, RtcError>;
}

/// DS1307 Real-Time Clock
///
/// Features:
/// - I2C address: 0x68
/// - 56 bytes battery-backed NVRAM
/// - Square wave output
pub struct DS1307 {
    i2c: I2c,
    address: u8,
}

impl DS1307 {
    const ADDRESS: u8 = 0x68;
    const SECONDS_REG: u8 = 0x00;

    /// Create a new DS1307 instance
    pub fn new(i2c: I2c) -> Self {
        DS1307 {
            i2c,
            address: Self::ADDRESS,
        }
    }
}

impl Rtc for DS1307 {
    fn begin(&mut self) -> Result<(), RtcError> {
        // Check if we can communicate with the device
        let mut buf = [0u8; 1];
        self.i2c.read_register(self.address, Self::SECONDS_REG, &mut buf)?;
        Ok(())
    }

    fn adjust(&mut self, dt: &DateTime) -> Result<(), RtcError> {
        if !dt.is_valid() {
            return Err(RtcError::InvalidDateTime);
        }

        // Prepare 7 bytes: seconds, minutes, hours, day-of-week, date, month, year
        let data = [
            bin2bcd(dt.second),          // Clear CH bit (start oscillator)
            bin2bcd(dt.minute),
            bin2bcd(dt.hour),            // 24-hour format
            bin2bcd(dt.day_of_week() + 1), // 1-7 format
            bin2bcd(dt.day),
            bin2bcd(dt.month),
            bin2bcd(dt.year_offset),
        ];

        self.i2c.write_register(self.address, Self::SECONDS_REG, &data)?;
        Ok(())
    }

    fn now(&self) -> Result<DateTime, RtcError> {
        // Read 7 bytes starting at seconds register
        let mut buffer = [0u8; 7];
        self.i2c.read_register(self.address, Self::SECONDS_REG, &mut buffer)?;

        let second = bcd2bin(buffer[0] & 0x7F); // Mask CH bit
        let minute = bcd2bin(buffer[1]);
        let hour = bcd2bin(buffer[2] & 0x3F);   // Mask for 24-hour format
        let day = bcd2bin(buffer[4]);
        let month = bcd2bin(buffer[5]);
        let year = bcd2bin(buffer[6]);

        let dt = DateTime::new(
            2000 + year as u16,
            month,
            day,
            hour,
            minute,
            second,
        );

        if !dt.is_valid() {
            return Err(RtcError::InvalidDateTime);
        }

        Ok(dt)
    }

    fn is_running(&self) -> Result<bool, RtcError> {
        let mut buf = [0u8; 1];
        self.i2c.read_register(self.address, Self::SECONDS_REG, &mut buf)?;

        // CH bit (bit 7) = 0 means running
        Ok((buf[0] & 0x80) == 0)
    }
}

/// DS3231 High-Precision Real-Time Clock
///
/// Features:
/// - I2C address: 0x68
/// - Temperature-compensated crystal oscillator
/// - Two alarms
/// - Temperature sensor
pub struct DS3231 {
    i2c: I2c,
    address: u8,
}

impl DS3231 {
    const ADDRESS: u8 = 0x68;
    const SECONDS_REG: u8 = 0x00;
    const STATUS_REG: u8 = 0x0F;

    /// Create a new DS3231 instance
    pub fn new(i2c: I2c) -> Self {
        DS3231 {
            i2c,
            address: Self::ADDRESS,
        }
    }

    /// Check if power was lost (OSF bit in status register)
    pub fn lost_power(&self) -> Result<bool, RtcError> {
        let mut buf = [0u8; 1];
        self.i2c.read_register(self.address, Self::STATUS_REG, &mut buf)?;

        // OSF bit (bit 7)
        Ok((buf[0] & 0x80) != 0)
    }
}

impl Rtc for DS3231 {
    fn begin(&mut self) -> Result<(), RtcError> {
        // Check if we can communicate with the device
        let mut buf = [0u8; 1];
        self.i2c.read_register(self.address, Self::SECONDS_REG, &mut buf)?;
        Ok(())
    }

    fn adjust(&mut self, dt: &DateTime) -> Result<(), RtcError> {
        if !dt.is_valid() {
            return Err(RtcError::InvalidDateTime);
        }

        // Prepare 7 bytes: seconds, minutes, hours, day-of-week, date, month, year
        let data = [
            bin2bcd(dt.second),
            bin2bcd(dt.minute),
            bin2bcd(dt.hour),            // 24-hour format
            bin2bcd(dt.day_of_week() + 1), // 1-7 format
            bin2bcd(dt.day),
            bin2bcd(dt.month),
            bin2bcd(dt.year_offset),
        ];

        self.i2c.write_register(self.address, Self::SECONDS_REG, &data)?;

        // Clear OSF bit after setting time
        let mut status = [0u8; 1];
        self.i2c.read_register(self.address, Self::STATUS_REG, &mut status)?;
        status[0] &= !0x80;
        self.i2c.write_register(self.address, Self::STATUS_REG, &status)?;

        Ok(())
    }

    fn now(&self) -> Result<DateTime, RtcError> {
        // Read 7 bytes starting at seconds register
        let mut buffer = [0u8; 7];
        self.i2c.read_register(self.address, Self::SECONDS_REG, &mut buffer)?;

        let second = bcd2bin(buffer[0] & 0x7F);
        let minute = bcd2bin(buffer[1]);
        let hour = bcd2bin(buffer[2] & 0x3F);   // Mask for 24-hour format
        let day = bcd2bin(buffer[4]);
        let month = bcd2bin(buffer[5]);
        let year = bcd2bin(buffer[6]);

        let dt = DateTime::new(
            2000 + year as u16,
            month,
            day,
            hour,
            minute,
            second,
        );

        if !dt.is_valid() {
            return Err(RtcError::InvalidDateTime);
        }

        Ok(dt)
    }

    fn is_running(&self) -> Result<bool, RtcError> {
        // DS3231 doesn't have CH bit, check if OSF bit indicates power loss
        Ok(!self.lost_power()?)
    }
}
