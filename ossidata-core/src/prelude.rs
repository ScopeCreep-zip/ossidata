//! Prelude module for convenient imports
//!
//! This module re-exports commonly used types and traits.

pub use crate::gpio::{Pin, mode};
pub use embedded_hal::digital::{OutputPin, InputPin};