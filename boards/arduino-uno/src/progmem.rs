//! PROGMEM support for storing data in flash memory
//!
//! This module provides Arduino-compatible PROGMEM macros for storing
//! constant data in flash memory instead of RAM, which is critical on
//! AVR microcontrollers with limited SRAM.

use core::ptr::read_volatile;

/// A string stored in program memory (flash)
///
/// This is equivalent to Arduino's F() macro and PROGMEM strings.
/// Data is stored in flash and read byte-by-byte as needed.
#[repr(C)]
pub struct FlashString {
    ptr: *const u8,
    len: usize,
}

impl FlashString {
    /// Create a new flash string from a static string
    ///
    /// # Safety
    /// The pointer must point to valid flash memory
    #[inline]
    pub const unsafe fn new(ptr: *const u8, len: usize) -> Self {
        Self { ptr, len }
    }

    /// Get the length of the flash string
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Check if the flash string is empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Read a byte from flash memory at the given index
    ///
    /// Returns None if index is out of bounds
    pub fn byte_at(&self, index: usize) -> Option<u8> {
        if index >= self.len {
            None
        } else {
            unsafe {
                Some(read_volatile(self.ptr.add(index)))
            }
        }
    }

    /// Iterate over bytes in the flash string
    pub fn bytes(&self) -> FlashStringIter {
        FlashStringIter {
            flash_str: self,
            pos: 0,
        }
    }

    /// Copy the flash string to a RAM buffer
    ///
    /// Returns the number of bytes copied (up to buffer.len())
    pub fn copy_to_slice(&self, buffer: &mut [u8]) -> usize {
        let copy_len = core::cmp::min(self.len, buffer.len());
        for i in 0..copy_len {
            buffer[i] = unsafe { read_volatile(self.ptr.add(i)) };
        }
        copy_len
    }
}

/// Iterator over bytes in a FlashString
pub struct FlashStringIter<'a> {
    flash_str: &'a FlashString,
    pos: usize,
}

impl<'a> Iterator for FlashStringIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let byte = self.flash_str.byte_at(self.pos)?;
        self.pos += 1;
        Some(byte)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.flash_str.len.saturating_sub(self.pos);
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for FlashStringIter<'a> {}

/// Macro to create a flash string (equivalent to Arduino's F() macro)
///
/// # Examples
/// ```no_run
/// use arduino_uno::F;
///
/// let message = F!("Hello from flash!");
/// serial.write_flash_str(&message);
/// ```
#[macro_export]
macro_rules! F {
    ($s:expr) => {{
        #[link_section = ".progmem.data"]
        static DATA: &[u8] = $s.as_bytes();
        unsafe {
            $crate::FlashString::new(
                DATA.as_ptr(),
                DATA.len(),
            )
        }
    }};
}

/// Read a byte from program memory
///
/// This is equivalent to Arduino's `pgm_read_byte()` macro.
///
/// # Safety
/// The pointer must point to valid flash memory
#[inline]
pub unsafe fn pgm_read_byte(ptr: *const u8) -> u8 {
    read_volatile(ptr)
}

/// Read a word (u16) from program memory
///
/// This is equivalent to Arduino's `pgm_read_word()` macro.
///
/// # Safety
/// The pointer must point to valid flash memory and be properly aligned
#[inline]
pub unsafe fn pgm_read_word(ptr: *const u16) -> u16 {
    read_volatile(ptr)
}

/// Read a dword (u32) from program memory
///
/// This is equivalent to Arduino's `pgm_read_dword()` macro.
///
/// # Safety
/// The pointer must point to valid flash memory and be properly aligned
#[inline]
pub unsafe fn pgm_read_dword(ptr: *const u32) -> u32 {
    read_volatile(ptr)
}

/// Read a float from program memory
///
/// This is equivalent to Arduino's `pgm_read_float()` macro.
///
/// # Safety
/// The pointer must point to valid flash memory and be properly aligned
#[inline]
pub unsafe fn pgm_read_float(ptr: *const f32) -> f32 {
    read_volatile(ptr)
}

/// Read a pointer from program memory
///
/// This is equivalent to Arduino's `pgm_read_ptr()` macro.
///
/// # Safety
/// The pointer must point to valid flash memory and be properly aligned
#[inline]
pub unsafe fn pgm_read_ptr<T>(ptr: *const *const T) -> *const T {
    read_volatile(ptr)
}
