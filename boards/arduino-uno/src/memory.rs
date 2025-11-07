//! Memory inspection and utilities
//!
//! This module provides functions for inspecting memory usage on the Arduino Uno.
//! The ATmega328P has:
//! - 2KB SRAM (0x100-0x8FF)
//! - 32KB Flash (program memory)
//! - 1KB EEPROM
//!
//! Stack grows downward from RAMEND, heap grows upward from __heap_start

use core::ptr::read_volatile;

// ATmega328P memory boundaries
const RAMSTART: usize = 0x100;
const RAMEND: usize = 0x8FF;
const RAMSIZE: usize = 2048;  // 2KB

// External symbols provided by linker
extern "C" {
    static mut __heap_start: u8;
    static mut __heap_end: u8;
    static mut __data_start: u8;
    static mut __data_end: u8;
    static mut __bss_start: u8;
    static mut __bss_end: u8;
}

/// Get the amount of free RAM between the heap and stack
///
/// This is equivalent to Arduino's `freeMemory()` function.
/// Returns the number of bytes between the top of the heap and the stack pointer.
///
/// # Example
/// ```no_run
/// use arduino_uno::free_memory;
///
/// let free = free_memory();
/// ```
#[inline(never)]
pub fn free_memory() -> usize {
    let heap_end = core::ptr::addr_of!(__heap_end) as usize;
    let stack_ptr = get_stack_pointer();

    if stack_ptr > heap_end {
        stack_ptr - heap_end
    } else {
        0  // Stack and heap have collided!
    }
}

/// Get the current stack pointer value
///
/// # Example
/// ```no_run
/// use arduino_uno::get_stack_pointer;
///
/// let sp = get_stack_pointer();
/// ```
#[inline(always)]
pub fn get_stack_pointer() -> usize {
    let spl: u8;
    let sph: u8;
    unsafe {
        core::arch::asm!(
            "in {spl}, 0x3d",
            "in {sph}, 0x3e",
            spl = out(reg) spl,
            sph = out(reg) sph,
            options(nomem, nostack)
        );
        ((sph as usize) << 8) | (spl as usize)
    }
}

/// Get the size of the .data section (initialized global variables)
///
/// # Example
/// ```no_run
/// use arduino_uno::data_size;
///
/// let size = data_size();
/// ```
pub fn data_size() -> usize {
    let start = core::ptr::addr_of!(__data_start) as usize;
    let end = core::ptr::addr_of!(__data_end) as usize;
    end.saturating_sub(start)
}

/// Get the size of the .bss section (uninitialized global variables)
///
/// # Example
/// ```no_run
/// use arduino_uno::bss_size;
///
/// let size = bss_size();
/// ```
pub fn bss_size() -> usize {
    let start = core::ptr::addr_of!(__bss_start) as usize;
    let end = core::ptr::addr_of!(__bss_end) as usize;
    end.saturating_sub(start)
}

/// Get the start address of the heap
///
/// # Example
/// ```no_run
/// use arduino_uno::heap_start;
///
/// let addr = heap_start();
/// ```
pub fn heap_start() -> usize {
    core::ptr::addr_of!(__heap_start) as usize
}

/// Get the end address of the heap
///
/// # Example
/// ```no_run
/// use arduino_uno::heap_end;
///
/// let addr = heap_end();
/// ```
pub fn heap_end() -> usize {
    core::ptr::addr_of!(__heap_end) as usize
}

/// Get total RAM size in bytes
///
/// For Arduino Uno (ATmega328P), this is always 2048 bytes (2KB).
///
/// # Example
/// ```no_run
/// use arduino_uno::ram_size;
///
/// let size = ram_size();  // Returns 2048
/// ```
pub const fn ram_size() -> usize {
    RAMSIZE
}

/// Get RAM start address
///
/// For ATmega328P, this is 0x100.
pub const fn ram_start_address() -> usize {
    RAMSTART
}

/// Get RAM end address
///
/// For ATmega328P, this is 0x8FF.
pub const fn ram_end_address() -> usize {
    RAMEND
}

/// Memory statistics structure
#[derive(Debug, Clone, Copy)]
pub struct MemoryInfo {
    /// Total RAM size in bytes
    pub total_ram: usize,
    /// Free RAM between heap and stack
    pub free_ram: usize,
    /// Size of .data section (initialized globals)
    pub data_section: usize,
    /// Size of .bss section (uninitialized globals)
    pub bss_section: usize,
    /// Heap start address
    pub heap_start: usize,
    /// Heap end address
    pub heap_end: usize,
    /// Current stack pointer
    pub stack_pointer: usize,
}

/// Get comprehensive memory statistics
///
/// Returns a structure containing detailed memory usage information.
///
/// # Example
/// ```no_run
/// use arduino_uno::memory_info;
///
/// let info = memory_info();
/// // info.free_ram contains available memory
/// // info.data_section + info.bss_section = used globals
/// ```
pub fn memory_info() -> MemoryInfo {
    MemoryInfo {
        total_ram: ram_size(),
        free_ram: free_memory(),
        data_section: data_size(),
        bss_section: bss_size(),
        heap_start: heap_start(),
        heap_end: heap_end(),
        stack_pointer: get_stack_pointer(),
    }
}

/// Check if there's enough stack space remaining
///
/// # Arguments
/// * `min_bytes` - Minimum required free bytes
///
/// # Returns
/// `true` if there's at least `min_bytes` free, `false` otherwise
///
/// # Example
/// ```no_run
/// use arduino_uno::check_stack_space;
///
/// if !check_stack_space(100) {
///     // Running low on memory!
/// }
/// ```
pub fn check_stack_space(min_bytes: usize) -> bool {
    free_memory() >= min_bytes
}

/// Fill a memory region with a pattern (useful for detecting stack usage)
///
/// # Safety
/// This is unsafe because it directly writes to memory.
/// Only use on unused stack/heap space.
///
/// # Arguments
/// * `start` - Start address
/// * `len` - Number of bytes to fill
/// * `pattern` - Byte pattern to write
pub unsafe fn fill_memory(start: *mut u8, len: usize, pattern: u8) {
    for i in 0..len {
        start.add(i).write_volatile(pattern);
    }
}

/// Count how many bytes of a pattern remain (for stack high-water mark)
///
/// # Safety
/// This reads from potentially uninitialized memory.
///
/// # Arguments
/// * `start` - Start address
/// * `len` - Number of bytes to check
/// * `pattern` - Pattern to look for
///
/// # Returns
/// Number of bytes that still contain the pattern
pub unsafe fn count_pattern(start: *const u8, len: usize, pattern: u8) -> usize {
    let mut count = 0;
    for i in 0..len {
        if read_volatile(start.add(i)) == pattern {
            count += 1;
        } else {
            break;  // Stop at first non-pattern byte
        }
    }
    count
}
