//! Arduino-compatible constants
//!
//! This module provides constants that match the Arduino API,
//! making it easier to port Arduino code to Rust.

// Digital pin values
pub const HIGH: bool = true;
pub const LOW: bool = false;

// Pin modes (for documentation - we use type-safe modes instead)
// These are provided for compatibility but prefer using .into_output() etc.
pub const INPUT: u8 = 0;
pub const OUTPUT: u8 = 1;
pub const INPUT_PULLUP: u8 = 2;

// Mathematical constants
pub const PI: f32 = core::f32::consts::PI;
pub const HALF_PI: f32 = core::f32::consts::FRAC_PI_2;
pub const TWO_PI: f32 = core::f32::consts::TAU;
pub const EULER: f32 = core::f32::consts::E;
pub const DEG_TO_RAD: f32 = PI / 180.0;
pub const RAD_TO_DEG: f32 = 180.0 / PI;

// Bit order for shiftIn/shiftOut
pub const LSBFIRST: bool = false;
pub const MSBFIRST: bool = true;

// Interrupt modes
pub const CHANGE: u8 = 0;
pub const FALLING: u8 = 1;
pub const RISING: u8 = 2;
// Note: LOW mode exists in Arduino but conflicts with digital LOW
// For interrupts on LOW level, use the InterruptMode enum instead

// Number bases for Print methods
pub const DEC: u8 = 10;
pub const HEX: u8 = 16;
pub const OCT: u8 = 8;
pub const BIN: u8 = 2;
