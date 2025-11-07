//! Arduino-compatible String class with safe memory management
//!
//! This module provides a String implementation that matches Arduino's String API
//! but uses a fixed-capacity buffer to avoid heap fragmentation issues.
//!
//! Unlike Arduino's String which uses dynamic allocation with realloc(),
//! this implementation uses a stack-allocated buffer with a configurable maximum size.

use core::fmt;
use ufmt::uWrite;

/// Default maximum string capacity
pub const DEFAULT_STRING_CAPACITY: usize = 64;

/// A fixed-capacity string similar to Arduino's String class
///
/// This implementation provides Arduino-compatible String methods but uses
/// a fixed-size buffer to avoid heap fragmentation.
///
/// # Example
/// ```no_run
/// use arduino_uno::ArduinoString;
///
/// let mut s = ArduinoString::<64>::new();
/// s.push_str("Hello");
/// s.push(' ');
/// s.push_str("World");
/// ```
pub struct ArduinoString<const N: usize> {
    buffer: [u8; N],
    len: usize,
}

impl<const N: usize> ArduinoString<N> {
    /// Create a new empty string
    pub const fn new() -> Self {
        Self {
            buffer: [0; N],
            len: 0,
        }
    }

    /// Create a string from a string slice
    pub fn from_str(s: &str) -> Self {
        let mut string = Self::new();
        string.push_str(s);
        string
    }

    /// Create a string from an integer
    pub fn from_int(value: i32, base: u8) -> Self {
        let mut string = Self::new();
        string.concat_int(value, base);
        string
    }

    /// Create a string from an unsigned integer
    pub fn from_uint(value: u32, base: u8) -> Self {
        let mut string = Self::new();
        string.concat_uint(value, base);
        string
    }

    /// Create a string from a float
    pub fn from_float(value: f32, digits: u8) -> Self {
        let mut string = Self::new();
        string.concat_float(value, digits);
        string
    }

    /// Get the length of the string
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Check if the string is empty
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the capacity of the string
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Get the remaining capacity
    pub const fn remaining_capacity(&self) -> usize {
        N - self.len
    }

    /// Clear the string
    pub fn clear(&mut self) {
        self.len = 0;
        self.buffer[0] = 0;
    }

    /// Get the string as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer[..self.len]
    }

    /// Get the string as a str
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buffer[..self.len]).unwrap_or("")
    }

    /// Get a C-style null-terminated string pointer
    ///
    /// Note: This ensures the buffer is null-terminated
    pub fn c_str(&mut self) -> *const u8 {
        if self.len < N {
            self.buffer[self.len] = 0;
        }
        self.buffer.as_ptr()
    }

    /// Get character at index
    pub fn char_at(&self, index: usize) -> Option<char> {
        if index < self.len {
            Some(self.buffer[index] as char)
        } else {
            None
        }
    }

    /// Set character at index
    pub fn set_char_at(&mut self, index: usize, ch: char) -> bool {
        if index < self.len && (ch as u32) < 256 {
            self.buffer[index] = ch as u8;
            true
        } else {
            false
        }
    }

    /// Append a character
    pub fn push(&mut self, ch: char) -> bool {
        if (ch as u32) < 256 && self.len < N {
            self.buffer[self.len] = ch as u8;
            self.len += 1;
            true
        } else {
            false
        }
    }

    /// Append a string slice
    pub fn push_str(&mut self, s: &str) -> bool {
        let bytes = s.as_bytes();
        if self.len + bytes.len() <= N {
            self.buffer[self.len..self.len + bytes.len()].copy_from_slice(bytes);
            self.len += bytes.len();
            true
        } else {
            false
        }
    }

    /// Concatenate an integer with specified base
    pub fn concat_int(&mut self, value: i32, base: u8) -> bool {
        if value < 0 && base == 10 {
            self.push('-') && self.concat_uint((-value) as u32, base)
        } else {
            self.concat_uint(value as u32, base)
        }
    }

    /// Concatenate an unsigned integer with specified base
    pub fn concat_uint(&mut self, mut value: u32, base: u8) -> bool {
        if base < 2 || base > 16 {
            return false;
        }

        if value == 0 {
            return self.push('0');
        }

        let mut digits = [0u8; 33];
        let mut count = 0;

        while value > 0 {
            let digit = (value % base as u32) as u8;
            digits[count] = if digit < 10 {
                b'0' + digit
            } else {
                b'A' + (digit - 10)
            };
            value /= base as u32;
            count += 1;
        }

        // Reverse and append
        if self.len + count <= N {
            for i in 0..count {
                self.buffer[self.len + i] = digits[count - 1 - i];
            }
            self.len += count;
            true
        } else {
            false
        }
    }

    /// Concatenate a float with specified decimal places
    pub fn concat_float(&mut self, value: f32, digits: u8) -> bool {
        if value.is_nan() {
            return self.push_str("nan");
        }

        if value.is_infinite() {
            if value < 0.0 {
                self.push('-');
            }
            return self.push_str("inf");
        }

        let mut val = value;
        if val < 0.0 {
            if !self.push('-') {
                return false;
            }
            val = -val;
        }

        // Add rounding
        let mut rounding = 0.5;
        for _ in 0..digits {
            rounding /= 10.0;
        }
        val += rounding;

        let int_part = val as u32;
        if !self.concat_uint(int_part, 10) {
            return false;
        }

        if digits > 0 {
            if !self.push('.') {
                return false;
            }

            let mut frac = val - int_part as f32;
            for _ in 0..digits {
                frac *= 10.0;
                let digit = frac as u32;
                if !self.push(('0' as u8 + digit as u8) as char) {
                    return false;
                }
                frac -= digit as f32;
            }
        }

        true
    }

    /// Find the first occurrence of a character
    pub fn index_of(&self, ch: char) -> Option<usize> {
        self.index_of_from(ch, 0)
    }

    /// Find the first occurrence of a character starting from index
    pub fn index_of_from(&self, ch: char, from: usize) -> Option<usize> {
        for i in from..self.len {
            if self.buffer[i] == ch as u8 {
                return Some(i);
            }
        }
        None
    }

    /// Find the last occurrence of a character
    pub fn last_index_of(&self, ch: char) -> Option<usize> {
        for i in (0..self.len).rev() {
            if self.buffer[i] == ch as u8 {
                return Some(i);
            }
        }
        None
    }

    /// Check if string starts with a prefix
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.as_str().starts_with(prefix)
    }

    /// Check if string ends with a suffix
    pub fn ends_with(&self, suffix: &str) -> bool {
        self.as_str().ends_with(suffix)
    }

    /// Convert to lowercase
    pub fn to_lower_case(&mut self) {
        for i in 0..self.len {
            self.buffer[i] = self.buffer[i].to_ascii_lowercase();
        }
    }

    /// Convert to uppercase
    pub fn to_upper_case(&mut self) {
        for i in 0..self.len {
            self.buffer[i] = self.buffer[i].to_ascii_uppercase();
        }
    }

    /// Trim whitespace from both ends
    pub fn trim(&mut self) {
        // Trim start
        let mut start = 0;
        while start < self.len && (self.buffer[start] as char).is_whitespace() {
            start += 1;
        }

        // Trim end
        let mut end = self.len;
        while end > start && (self.buffer[end - 1] as char).is_whitespace() {
            end -= 1;
        }

        // Shift if needed
        if start > 0 {
            for i in 0..(end - start) {
                self.buffer[i] = self.buffer[start + i];
            }
        }
        self.len = end - start;
    }

    /// Remove characters from index to end
    pub fn remove(&mut self, index: usize) {
        if index < self.len {
            self.len = index;
        }
    }

    /// Remove range of characters
    pub fn remove_range(&mut self, start: usize, count: usize) {
        if start >= self.len {
            return;
        }

        let end = (start + count).min(self.len);
        let remaining = self.len - end;

        for i in 0..remaining {
            self.buffer[start + i] = self.buffer[end + i];
        }
        self.len = start + remaining;
    }

    /// Replace all occurrences of a character
    pub fn replace_char(&mut self, from: char, to: char) -> usize {
        let mut count = 0;
        if (to as u32) < 256 {
            for i in 0..self.len {
                if self.buffer[i] == from as u8 {
                    self.buffer[i] = to as u8;
                    count += 1;
                }
            }
        }
        count
    }

    /// Parse string as integer
    pub fn to_int(&self) -> i32 {
        let s = self.as_str().trim();
        let mut result = 0i32;
        let mut negative = false;
        let mut start = 0;

        if s.starts_with('-') {
            negative = true;
            start = 1;
        } else if s.starts_with('+') {
            start = 1;
        }

        for ch in s[start..].chars() {
            if let Some(digit) = ch.to_digit(10) {
                result = result.saturating_mul(10).saturating_add(digit as i32);
            } else {
                break;
            }
        }

        if negative { -result } else { result }
    }

    /// Parse string as float
    pub fn to_float(&self) -> f32 {
        // Simple float parsing
        let s = self.as_str().trim();
        let mut result = 0.0f32;
        let mut negative = false;
        let mut decimal_places = 0i32;
        let mut after_decimal = false;
        let mut start = 0;

        if s.starts_with('-') {
            negative = true;
            start = 1;
        } else if s.starts_with('+') {
            start = 1;
        }

        for ch in s[start..].chars() {
            if ch == '.' {
                after_decimal = true;
            } else if let Some(digit) = ch.to_digit(10) {
                result = result * 10.0 + digit as f32;
                if after_decimal {
                    decimal_places += 1;
                }
            } else {
                break;
            }
        }

        for _ in 0..decimal_places {
            result /= 10.0;
        }

        if negative { -result } else { result }
    }

    /// Compare with another string (case-sensitive)
    pub fn equals(&self, other: &str) -> bool {
        self.as_str() == other
    }

    /// Compare with another string (case-insensitive)
    pub fn equals_ignore_case(&self, other: &str) -> bool {
        self.as_str().eq_ignore_ascii_case(other)
    }

    /// Compare strings lexicographically
    pub fn compare_to(&self, other: &str) -> core::cmp::Ordering {
        self.as_str().cmp(other)
    }
}

impl<const N: usize> Default for ArduinoString<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> fmt::Debug for ArduinoString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.as_str())
    }
}

impl<const N: usize> fmt::Display for ArduinoString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<const N: usize> uWrite for ArduinoString<N> {
    type Error = ();

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        if self.push_str(s) {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<const N: usize> PartialEq for ArduinoString<N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const N: usize> PartialEq<str> for ArduinoString<N> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<const N: usize> PartialEq<&str> for ArduinoString<N> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<const N: usize> Eq for ArduinoString<N> {}

impl<const N: usize> PartialOrd for ArduinoString<N> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.as_str().cmp(other.as_str()))
    }
}

impl<const N: usize> Ord for ArduinoString<N> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

/// Type alias for String with default capacity
pub type String = ArduinoString<DEFAULT_STRING_CAPACITY>;
