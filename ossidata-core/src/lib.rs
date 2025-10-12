//! Ossidata Core - Hardware abstraction layer for Arduino boards
//!
//! This crate provides the core abstractions and traits used across
//! all Ossidata board implementations.

#![no_std]
#![warn(missing_docs)]

pub mod gpio;
pub mod prelude;

/// Re-export embedded-hal traits
pub use embedded_hal;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");