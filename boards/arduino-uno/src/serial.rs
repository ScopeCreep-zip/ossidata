//! UART/Serial communication implementation for Arduino Uno
//!
//! The ATmega328P has one hardware UART (USART0) connected to pins 0 (RX) and 1 (TX).
//! These pins are also connected to the USB-to-serial converter on the Arduino board.

use core::ptr::{read_volatile, write_volatile};
use core::cell::Cell;
use critical_section::Mutex;
use ufmt::uWrite;

// USART0 register addresses for ATmega328P
const UDR0: *mut u8 = 0xC6 as *mut u8;   // USART Data Register
const UCSR0A: *mut u8 = 0xC0 as *mut u8; // USART Control and Status Register A
const UCSR0B: *mut u8 = 0xC1 as *mut u8; // USART Control and Status Register B
const UCSR0C: *mut u8 = 0xC2 as *mut u8; // USART Control and Status Register C
const UBRR0L: *mut u8 = 0xC4 as *mut u8; // USART Baud Rate Register Low
const UBRR0H: *mut u8 = 0xC5 as *mut u8; // USART Baud Rate Register High

// Stream parsing constants
const NO_SKIP_CHAR: u8 = 1;  // For parseInt/parseFloat - don't skip any char

// Global timeout for stream operations (in milliseconds)
static STREAM_TIMEOUT: Mutex<Cell<u32>> = Mutex::new(Cell::new(1000));

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
    peek_byte: Option<u8>,  // Buffer for peek() operation
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

        Serial {
            peek_byte: None,
        }
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

    /// Check if the transmit buffer is ready for writing
    ///
    /// Returns true if the UART is ready to accept more data for transmission.
    /// This is equivalent to Arduino's Serial.availableForWrite().
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::Serial;
    ///
    /// let mut serial = Serial::new(9600);
    /// if serial.available_for_write() {
    ///     serial.write_byte(b'A');
    /// }
    /// ```
    pub fn available_for_write(&self) -> bool {
        unsafe {
            read_volatile(UCSR0A) & (1 << UDRE0) != 0
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

    /// Print an integer in a specified base (DEC, HEX, OCT, BIN)
    ///
    /// # Arguments
    /// * `value` - The number to print
    /// * `base` - The base (2=BIN, 8=OCT, 10=DEC, 16=HEX)
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::Serial;
    ///
    /// let mut serial = Serial::new(9600);
    /// serial.print_int(255, 16);  // Prints "FF"
    /// serial.print_int(255, 2);   // Prints "11111111"
    /// serial.print_int(255, 8);   // Prints "377"
    /// ```
    pub fn print_int(&mut self, value: i32, base: u8) {
        if value < 0 && base == 10 {
            self.write_byte(b'-');
            self.print_uint((-value) as u32, base);
        } else {
            self.print_uint(value as u32, base);
        }
    }

    /// Print an unsigned integer in a specified base
    pub fn print_uint(&mut self, mut value: u32, base: u8) {
        if base < 2 || base > 16 {
            return;
        }

        let mut buffer = [0u8; 33]; // Max binary representation + 1
        let mut i = 0;

        if value == 0 {
            self.write_byte(b'0');
            return;
        }

        while value > 0 {
            let digit = (value % base as u32) as u8;
            buffer[i] = if digit < 10 {
                b'0' + digit
            } else {
                b'A' + (digit - 10)
            };
            value /= base as u32;
            i += 1;
        }

        // Print in reverse order
        while i > 0 {
            i -= 1;
            self.write_byte(buffer[i]);
        }
    }

    /// Print a float with specified decimal places
    ///
    /// # Arguments
    /// * `value` - The floating point number to print
    /// * `digits` - Number of decimal places (default 2)
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::Serial;
    ///
    /// let mut serial = Serial::new(9600);
    /// serial.print_float(3.14159, 2);  // Prints "3.14"
    /// serial.print_float(3.14159, 4);  // Prints "3.1416"
    /// ```
    pub fn print_float(&mut self, value: f32, digits: u8) {
        if value.is_nan() {
            self.write_str("nan");
            return;
        }

        if value.is_infinite() {
            if value < 0.0 {
                self.write_byte(b'-');
            }
            self.write_str("inf");
            return;
        }

        // Handle negative
        let mut val = value;
        if val < 0.0 {
            self.write_byte(b'-');
            val = -val;
        }

        // Round to specified digits
        let mut rounding = 0.5;
        for _ in 0..digits {
            rounding /= 10.0;
        }
        val += rounding;

        // Extract integer part
        let int_part = val as u32;
        self.print_uint(int_part, 10);

        // Extract fractional part
        if digits > 0 {
            self.write_byte(b'.');

            let mut frac = val - int_part as f32;
            for _ in 0..digits {
                frac *= 10.0;
                let digit = frac as u32;
                self.write_byte(b'0' + (digit as u8));
                frac -= digit as f32;
            }
        }
    }

    /// Print integer followed by newline
    pub fn println_int(&mut self, value: i32, base: u8) {
        self.print_int(value, base);
        self.print_newline();
    }

    /// Print unsigned integer followed by newline
    pub fn println_uint(&mut self, value: u32, base: u8) {
        self.print_uint(value, base);
        self.print_newline();
    }

    /// Print float followed by newline
    pub fn println_float(&mut self, value: f32, digits: u8) {
        self.print_float(value, digits);
        self.print_newline();
    }

    /// Write a flash string (PROGMEM string) to serial
    ///
    /// This is equivalent to Arduino's `Serial.print(F("string"))`.
    /// It reads the string directly from flash memory, saving RAM.
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::{Serial, F};
    ///
    /// let mut serial = Serial::new(9600);
    /// serial.write_flash_str(&F!("Hello from flash!"));
    /// ```
    pub fn write_flash_str(&mut self, flash_str: &crate::FlashString) {
        for byte in flash_str.bytes() {
            self.write_byte(byte);
        }
    }

    /// Write a flash string followed by newline
    pub fn writeln_flash_str(&mut self, flash_str: &crate::FlashString) {
        self.write_flash_str(flash_str);
        self.print_newline();
    }

    /// Wait for transmission to complete
    ///
    /// This ensures all data has been physically transmitted from the UART
    /// before returning. Useful before entering sleep modes or critical timing sections.
    pub fn flush(&mut self) {
        unsafe {
            // Wait for transmit complete flag
            while read_volatile(UCSR0A) & (1 << TXC0) == 0 {}
            // Clear the flag by writing 1 to it
            write_volatile(UCSR0A, 1 << TXC0);
        }
    }

    // ===== Stream Methods =====

    /// Peek at the next byte without removing it from the buffer
    ///
    /// Returns the next byte available or -1 if no data available.
    /// Unlike read_byte(), this does not remove the byte from the stream.
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::Serial;
    ///
    /// let mut serial = Serial::new(9600);
    /// if let Some(byte) = serial.peek() {
    ///     // Look at byte without consuming it
    ///     if byte == b'A' {
    ///         serial.read_byte(); // Now consume it
    ///     }
    /// }
    /// ```
    pub fn peek(&mut self) -> Option<u8> {
        if let Some(byte) = self.peek_byte {
            Some(byte)
        } else if self.available() {
            let byte = unsafe { read_volatile(UDR0) };
            self.peek_byte = Some(byte);
            Some(byte)
        } else {
            None
        }
    }

    /// Set the timeout for stream operations in milliseconds
    ///
    /// This timeout is used by parseInt(), parseFloat(), readBytes(), etc.
    /// Default is 1000ms (1 second).
    ///
    /// # Arguments
    /// * `timeout_ms` - Timeout in milliseconds
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::Serial;
    ///
    /// let mut serial = Serial::new(9600);
    /// serial.set_timeout(5000);  // 5 second timeout
    /// ```
    pub fn set_timeout(&mut self, timeout_ms: u32) {
        critical_section::with(|cs| {
            STREAM_TIMEOUT.borrow(cs).set(timeout_ms);
        });
    }

    /// Get the current timeout for stream operations
    pub fn get_timeout(&self) -> u32 {
        critical_section::with(|cs| {
            STREAM_TIMEOUT.borrow(cs).get()
        })
    }

    /// Read a byte with timeout support
    ///
    /// Returns None if timeout expires before data is available
    fn read_byte_timeout(&mut self) -> Option<u8> {
        // Check if we have a peeked byte
        if let Some(byte) = self.peek_byte.take() {
            return Some(byte);
        }

        let timeout = self.get_timeout();
        let start = crate::millis();

        while !self.available() {
            if crate::millis() - start >= timeout {
                return None;  // Timeout
            }
        }

        Some(self.read_byte())
    }

    /// Parse an integer from the stream
    ///
    /// Reads characters until a non-digit is found or timeout occurs.
    /// Leading whitespace is skipped. Returns None on timeout or if no digits found.
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::Serial;
    ///
    /// let mut serial = Serial::new(9600);
    /// if let Some(value) = serial.parse_int() {
    ///     // Use the parsed integer
    /// }
    /// ```
    pub fn parse_int(&mut self) -> Option<i32> {
        self.parse_int_internal(NO_SKIP_CHAR)
    }

    fn parse_int_internal(&mut self, skip_char: u8) -> Option<i32> {
        let mut is_negative = false;
        let mut value: i32 = 0;
        let mut found_digit = false;

        loop {
            let byte = self.read_byte_timeout()?;

            // Skip specified character if any
            if skip_char != NO_SKIP_CHAR && byte == skip_char {
                continue;
            }

            // Skip whitespace at start
            if !found_digit && (byte == b' ' || byte == b'\t' || byte == b'\r' || byte == b'\n') {
                continue;
            }

            // Handle negative sign
            if !found_digit && byte == b'-' {
                is_negative = true;
                continue;
            }

            // Handle positive sign
            if !found_digit && byte == b'+' {
                continue;
            }

            // Parse digit
            if byte >= b'0' && byte <= b'9' {
                found_digit = true;
                value = value.saturating_mul(10).saturating_add((byte - b'0') as i32);
            } else if found_digit {
                // Non-digit after digits - put it back for next read
                self.peek_byte = Some(byte);
                break;
            } else {
                // Non-digit before any digits - no valid integer
                return None;
            }
        }

        if !found_digit {
            None
        } else if is_negative {
            Some(-value)
        } else {
            Some(value)
        }
    }

    /// Parse a floating point number from the stream
    ///
    /// Reads characters until a non-numeric character is found or timeout occurs.
    /// Handles decimal points and leading signs. Returns None on timeout or invalid format.
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::Serial;
    ///
    /// let mut serial = Serial::new(9600);
    /// if let Some(value) = serial.parse_float() {
    ///     // Use the parsed float
    /// }
    /// ```
    pub fn parse_float(&mut self) -> Option<f32> {
        let mut is_negative = false;
        let mut value: f32 = 0.0;
        let mut fraction: f32 = 1.0;
        let mut found_digit = false;
        let mut is_fraction = false;

        loop {
            let byte = self.read_byte_timeout()?;

            // Skip whitespace at start
            if !found_digit && (byte == b' ' || byte == b'\t' || byte == b'\r' || byte == b'\n') {
                continue;
            }

            // Handle negative sign
            if !found_digit && byte == b'-' {
                is_negative = true;
                continue;
            }

            // Handle positive sign
            if !found_digit && byte == b'+' {
                continue;
            }

            // Handle decimal point
            if byte == b'.' && !is_fraction {
                is_fraction = true;
                continue;
            }

            // Parse digit
            if byte >= b'0' && byte <= b'9' {
                found_digit = true;
                let digit = (byte - b'0') as f32;

                if is_fraction {
                    fraction *= 0.1;
                    value += digit * fraction;
                } else {
                    value = value * 10.0 + digit;
                }
            } else if found_digit {
                // Non-digit after digits - put it back for next read
                self.peek_byte = Some(byte);
                break;
            } else {
                // Non-digit before any digits - no valid float
                return None;
            }
        }

        if !found_digit {
            None
        } else if is_negative {
            Some(-value)
        } else {
            Some(value)
        }
    }

    /// Read bytes into a buffer
    ///
    /// Reads up to `length` bytes into the buffer. Returns the number of bytes read.
    /// Will timeout according to the timeout set by set_timeout().
    ///
    /// # Arguments
    /// * `buffer` - Buffer to read into
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::Serial;
    ///
    /// let mut serial = Serial::new(9600);
    /// let mut buffer = [0u8; 10];
    /// let count = serial.read_bytes(&mut buffer);
    /// ```
    pub fn read_bytes(&mut self, buffer: &mut [u8]) -> usize {
        let mut count = 0;

        for i in 0..buffer.len() {
            if let Some(byte) = self.read_byte_timeout() {
                buffer[i] = byte;
                count += 1;
            } else {
                break;  // Timeout
            }
        }

        count
    }

    /// Read bytes until a terminator is found
    ///
    /// Reads bytes into buffer until the terminator character is found,
    /// buffer is full, or timeout occurs. The terminator is not included
    /// in the buffer. Returns the number of bytes read.
    ///
    /// # Arguments
    /// * `terminator` - Character to stop reading at
    /// * `buffer` - Buffer to read into
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::Serial;
    ///
    /// let mut serial = Serial::new(9600);
    /// let mut buffer = [0u8; 64];
    /// let count = serial.read_bytes_until(b'\n', &mut buffer);
    /// ```
    pub fn read_bytes_until(&mut self, terminator: u8, buffer: &mut [u8]) -> usize {
        let mut count = 0;

        for i in 0..buffer.len() {
            if let Some(byte) = self.read_byte_timeout() {
                if byte == terminator {
                    break;  // Found terminator
                }
                buffer[i] = byte;
                count += 1;
            } else {
                break;  // Timeout
            }
        }

        count
    }

    /// Search for a target sequence in the stream
    ///
    /// Reads data from the stream until the target is found or timeout occurs.
    /// Returns true if target was found, false on timeout.
    ///
    /// # Arguments
    /// * `target` - Byte sequence to search for
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::Serial;
    ///
    /// let mut serial = Serial::new(9600);
    /// if serial.find(b"OK") {
    ///     // Found "OK" in stream
    /// }
    /// ```
    pub fn find(&mut self, target: &[u8]) -> bool {
        if target.is_empty() {
            return true;
        }

        let mut match_index = 0;

        loop {
            if let Some(byte) = self.read_byte_timeout() {
                if byte == target[match_index] {
                    match_index += 1;
                    if match_index >= target.len() {
                        return true;  // Found complete match
                    }
                } else {
                    match_index = 0;  // Reset on mismatch
                }
            } else {
                return false;  // Timeout
            }
        }
    }

    /// Search for a target sequence, but stop at a terminator
    ///
    /// Reads data from the stream until the target is found, terminator is found,
    /// or timeout occurs. Returns true if target was found before terminator.
    ///
    /// # Arguments
    /// * `target` - Byte sequence to search for
    /// * `terminator` - Byte sequence that stops the search
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::Serial;
    ///
    /// let mut serial = Serial::new(9600);
    /// if serial.find_until(b"OK", b"\n") {
    ///     // Found "OK" before newline
    /// }
    /// ```
    pub fn find_until(&mut self, target: &[u8], terminator: &[u8]) -> bool {
        if target.is_empty() {
            return true;
        }

        let mut target_index = 0;
        let mut term_index = 0;

        loop {
            if let Some(byte) = self.read_byte_timeout() {
                // Check target match
                if byte == target[target_index] {
                    target_index += 1;
                    if target_index >= target.len() {
                        return true;  // Found target
                    }
                } else {
                    target_index = 0;
                }

                // Check terminator match
                if !terminator.is_empty() && byte == terminator[term_index] {
                    term_index += 1;
                    if term_index >= terminator.len() {
                        return false;  // Hit terminator before finding target
                    }
                } else {
                    term_index = 0;
                }
            } else {
                return false;  // Timeout
            }
        }
    }

    /// Read all available characters into a String
    ///
    /// Reads characters from the serial buffer until no more data is available
    /// or the string capacity is reached.
    ///
    /// # Example
    /// ```no_run
    /// use arduino_uno::Serial;
    ///
    /// let mut serial = Serial::new(9600);
    /// let data = serial.read_string::<64>();
    /// ```
    pub fn read_string<const N: usize>(&mut self) -> crate::ArduinoString<N> {
        let mut result = crate::ArduinoString::<N>::new();

        loop {
            if !self.available() {
                break;
            }

            let byte = self.read_byte();
            if !result.push(byte as char) {
                break;  // String full
            }
        }

        result
    }

    /// Read characters into a String until terminator character is found
    ///
    /// Reads characters from the serial buffer until the terminator is encountered,
    /// timeout occurs, or the string capacity is reached.
    ///
    /// # Arguments
    /// * `terminator` - Character that marks the end of the string
    ///
    /// # Example
    /// ```no_run
    /// use arduino_uno::Serial;
    ///
    /// let mut serial = Serial::new(9600);
    /// let line = serial.read_string_until::<64>('\n');
    /// ```
    pub fn read_string_until<const N: usize>(&mut self, terminator: char) -> crate::ArduinoString<N> {
        let mut result = crate::ArduinoString::<N>::new();
        let timeout = self.get_timeout();
        let start = crate::millis();

        loop {
            // Check timeout
            if crate::millis() - start >= timeout {
                break;
            }

            if !self.available() {
                continue;
            }

            let byte = self.read_byte();

            if byte == terminator as u8 {
                break;
            }

            if !result.push(byte as char) {
                break;  // String full
            }
        }

        result
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