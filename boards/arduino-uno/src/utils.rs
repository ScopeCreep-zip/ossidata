//! Utility functions compatible with Arduino API
//!
//! This module provides common utility functions that match the Arduino API,
//! including mathematical operations, random number generation, and value mapping.

use core::cell::Cell;
use critical_section::Mutex;

/// Random number generator seed
static RANDOM_SEED: Mutex<Cell<u32>> = Mutex::new(Cell::new(1));

/// Re-maps a number from one range to another
///
/// This is equivalent to Arduino's `map()` function. The value is mapped
/// proportionally from the input range to the output range.
///
/// # Arguments
/// * `value` - The number to map
/// * `in_min` - Lower bound of the input range
/// * `in_max` - Upper bound of the input range
/// * `out_min` - Lower bound of the output range
/// * `out_max` - Upper bound of the output range
///
/// # Examples
/// ```no_run
/// use arduino_uno::map;
///
/// // Map ADC reading (0-1023) to PWM output (0-255)
/// let pwm_value = map(512, 0, 1023, 0, 255);
/// assert_eq!(pwm_value, 127);
/// ```
///
/// # Note
/// Does not constrain values to within the output range. Use `constrain()`
/// if you need to ensure the output stays within bounds.
pub fn map(value: i32, in_min: i32, in_max: i32, out_min: i32, out_max: i32) -> i32 {
    ((value - in_min) as i64 * (out_max - out_min) as i64 / (in_max - in_min) as i64) as i32
        + out_min
}

/// Constrains a number to be within a range
///
/// This is equivalent to Arduino's `constrain()` function.
///
/// # Arguments
/// * `value` - The number to constrain
/// * `min` - Lower bound of the range
/// * `max` - Upper bound of the range
///
/// # Examples
/// ```no_run
/// use arduino_uno::constrain;
///
/// let val = constrain(150, 0, 100);  // Returns 100
/// let val = constrain(-10, 0, 100);  // Returns 0
/// let val = constrain(50, 0, 100);   // Returns 50
/// ```
pub fn constrain<T: Ord>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Returns the minimum of two values
///
/// # Examples
/// ```no_run
/// use arduino_uno::min;
///
/// let result = min(10, 20);  // Returns 10
/// ```
pub fn min<T: Ord>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

/// Returns the maximum of two values
///
/// # Examples
/// ```no_run
/// use arduino_uno::max;
///
/// let result = max(10, 20);  // Returns 20
/// ```
pub fn max<T: Ord>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}

/// Returns the absolute value
///
/// # Examples
/// ```no_run
/// use arduino_uno::abs;
///
/// let result = abs(-42);  // Returns 42
/// ```
pub fn abs(value: i32) -> i32 {
    if value < 0 {
        -value
    } else {
        value
    }
}

/// Calculates the square of a number
///
/// # Examples
/// ```no_run
/// use arduino_uno::sq;
///
/// let result = sq(5);  // Returns 25
/// ```
pub fn sq(value: i32) -> i32 {
    value * value
}

/// Initializes the pseudo-random number generator
///
/// Seeds the random number generator with the specified value.
/// Use this to create different sequences of random numbers.
///
/// # Arguments
/// * `seed` - The seed value for the random number generator
///
/// # Examples
/// ```no_run
/// use arduino_uno::{random_seed, random};
///
/// random_seed(42);
/// let value = random(0, 100);
/// ```
///
/// # Note
/// A common technique is to seed with an analog reading from an unconnected pin:
/// ```no_run
/// use arduino_uno::{Adc, random_seed};
///
/// let mut adc = Adc::new();
/// random_seed(adc.read(0) as u32);
/// ```
pub fn random_seed(seed: u32) {
    critical_section::with(|cs| {
        RANDOM_SEED.borrow(cs).set(if seed == 0 { 1 } else { seed });
    });
}

/// Generates a pseudo-random number
///
/// Returns a random number within the specified range using a simple
/// Linear Congruential Generator (LCG) algorithm.
///
/// # Arguments
/// * `min` - Lower bound (inclusive)
/// * `max` - Upper bound (exclusive)
///
/// # Examples
/// ```no_run
/// use arduino_uno::random;
///
/// let dice_roll = random(1, 7);  // Returns 1-6
/// let coin_flip = random(0, 2);  // Returns 0 or 1
/// ```
///
/// # Note
/// This is a simple pseudo-random number generator suitable for
/// games and non-cryptographic applications. Not suitable for
/// security-critical applications.
pub fn random(min: i32, max: i32) -> i32 {
    if min >= max {
        return min;
    }

    // Linear Congruential Generator (LCG)
    // Using constants from Numerical Recipes
    let next = critical_section::with(|cs| {
        let seed = RANDOM_SEED.borrow(cs).get();
        let next = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        RANDOM_SEED.borrow(cs).set(next);
        next
    });

    // Scale to range
    let range = (max - min) as u32;
    min + (next % range) as i32
}

/// Single-argument random function (0 to max-1)
///
/// # Arguments
/// * `max` - Upper bound (exclusive)
///
/// # Examples
/// ```no_run
/// use arduino_uno::random_max;
///
/// let value = random_max(100);  // Returns 0-99
/// ```
pub fn random_max(max: i32) -> i32 {
    random(0, max)
}

/// Converts degrees to radians
///
/// # Examples
/// ```no_run
/// use arduino_uno::radians;
///
/// let rad = radians(180.0);  // Returns π
/// ```
pub fn radians(degrees: f32) -> f32 {
    degrees * 0.017453292519943295  // π / 180
}

/// Converts radians to degrees
///
/// # Examples
/// ```no_run
/// use arduino_uno::degrees;
///
/// let deg = degrees(3.14159);  // Returns ~180
/// ```
pub fn degrees(radians: f32) -> f32 {
    radians * 57.29577951308232  // 180 / π
}

// Bit manipulation helpers

/// Reads a bit from a number
///
/// # Arguments
/// * `value` - The number to read from
/// * `bit` - The bit position (0-31)
///
/// # Examples
/// ```no_run
/// use arduino_uno::bit_read;
///
/// let value = 0b1010;
/// assert_eq!(bit_read(value, 0), 0);
/// assert_eq!(bit_read(value, 1), 1);
/// ```
pub fn bit_read(value: u32, bit: u8) -> u8 {
    ((value >> bit) & 1) as u8
}

/// Sets a bit in a number
///
/// # Arguments
/// * `value` - The number to modify
/// * `bit` - The bit position (0-31)
///
/// # Examples
/// ```no_run
/// use arduino_uno::bit_set;
///
/// let value = 0b0000;
/// let result = bit_set(value, 2);  // Returns 0b0100
/// ```
pub fn bit_set(value: u32, bit: u8) -> u32 {
    value | (1 << bit)
}

/// Clears a bit in a number
///
/// # Arguments
/// * `value` - The number to modify
/// * `bit` - The bit position (0-31)
///
/// # Examples
/// ```no_run
/// use arduino_uno::bit_clear;
///
/// let value = 0b1111;
/// let result = bit_clear(value, 2);  // Returns 0b1011
/// ```
pub fn bit_clear(value: u32, bit: u8) -> u32 {
    value & !(1 << bit)
}

/// Toggles a bit in a number
///
/// # Arguments
/// * `value` - The number to modify
/// * `bit` - The bit position (0-31)
///
/// # Examples
/// ```no_run
/// use arduino_uno::bit_toggle;
///
/// let value = 0b1010;
/// let result = bit_toggle(value, 0);  // Returns 0b1011
/// ```
pub fn bit_toggle(value: u32, bit: u8) -> u32 {
    value ^ (1 << bit)
}

/// Writes a bit in a number
///
/// # Arguments
/// * `value` - The number to modify
/// * `bit` - The bit position (0-31)
/// * `bit_value` - 0 to clear, non-zero to set
///
/// # Examples
/// ```no_run
/// use arduino_uno::bit_write;
///
/// let value = 0b0000;
/// let result = bit_write(value, 2, 1);  // Returns 0b0100
/// ```
pub fn bit_write(value: u32, bit: u8, bit_value: u8) -> u32 {
    if bit_value != 0 {
        bit_set(value, bit)
    } else {
        bit_clear(value, bit)
    }
}

/// Returns the low byte of a value
///
/// # Examples
/// ```no_run
/// use arduino_uno::low_byte;
///
/// let result = low_byte(0x1234);  // Returns 0x34
/// ```
pub fn low_byte(value: u16) -> u8 {
    (value & 0xFF) as u8
}

/// Returns the high byte of a value
///
/// # Examples
/// ```no_run
/// use arduino_uno::high_byte;
///
/// let result = high_byte(0x1234);  // Returns 0x12
/// ```
pub fn high_byte(value: u16) -> u8 {
    ((value >> 8) & 0xFF) as u8
}

/// Creates a word from two bytes
///
/// # Examples
/// ```no_run
/// use arduino_uno::make_word;
///
/// let result = make_word(0x12, 0x34);  // Returns 0x1234
/// ```
pub fn make_word(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | (low as u16)
}

/// Returns a value with bit n set (2^n)
///
/// # Examples
/// ```no_run
/// use arduino_uno::bit;
///
/// let value = bit(3);  // Returns 8 (0b1000)
/// let mask = bit(0) | bit(2);  // Returns 5 (0b0101)
/// ```
pub fn bit(n: u8) -> u32 {
    1u32 << n
}

/// Rounds a floating point number to the nearest integer
///
/// # Examples
/// ```no_run
/// use arduino_uno::round;
///
/// let result = round(3.7);  // Returns 4.0
/// let result = round(3.2);  // Returns 3.0
/// let result = round(-2.5); // Returns -2.0 (rounds to even)
/// ```
pub fn round(value: f32) -> f32 {
    if value >= 0.0 {
        (value + 0.5) as i32 as f32
    } else {
        (value - 0.5) as i32 as f32
    }
}

/// Enable global interrupts
///
/// This is equivalent to Arduino's `interrupts()` function.
/// Re-enables interrupts after they have been disabled with `noInterrupts()`.
///
/// # Safety
/// This function uses inline assembly to enable interrupts globally.
///
/// # Examples
/// ```no_run
/// use arduino_uno::{interrupts, noInterrupts};
///
/// noInterrupts();  // Disable interrupts
/// // Critical section code here
/// interrupts();    // Re-enable interrupts
/// ```
#[inline(always)]
pub fn interrupts() {
    unsafe {
        core::arch::asm!("sei", options(nomem, nostack));
    }
}

/// Disable global interrupts
///
/// This is equivalent to Arduino's `noInterrupts()` function.
/// Disables interrupts to protect critical sections of code.
///
/// # Safety
/// This function uses inline assembly to disable interrupts globally.
/// Interrupts should be re-enabled as soon as possible using `interrupts()`.
///
/// # Examples
/// ```no_run
/// use arduino_uno::{interrupts, noInterrupts};
///
/// noInterrupts();  // Disable interrupts
/// // Critical section - no interrupts will fire
/// interrupts();    // Re-enable interrupts
/// ```
#[inline(always)]
pub fn no_interrupts() {
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack));
    }
}

/// Cooperative yield function
///
/// This function can be overridden to implement cooperative multitasking.
/// By default, it does nothing. The delay() function calls this periodically
/// to allow background tasks to run.
///
/// # Examples
/// ```no_run
/// use arduino_uno::yield_now;
///
/// // Called automatically by delay(), or call manually:
/// loop {
///     // Do work
///     yield_now();  // Allow other tasks to run
/// }
/// ```
#[inline(always)]
pub fn yield_now() {
    // Empty by default - can be overridden for cooperative multitasking
}
